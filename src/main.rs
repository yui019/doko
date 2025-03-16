use std::thread;

use cursive::{
    Cursive, Printer, Vec2, View,
    reexports::crossbeam_channel::{self, Receiver},
    theme::Theme,
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout},
};
use doko::levenshtein;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

static mut ALL_FILES: Vec<String> = vec![];

fn get_filtered_files(query: &str) -> Vec<&'static String> {
    // this isn't actually unsafe :3
    #[allow(static_mut_refs)]
    unsafe {
        let filtered_files: Vec<&String> = ALL_FILES
            .par_iter()
            .filter(|&file| levenshtein(&file, &query) < 5)
            .collect();

        return filtered_files;
    }
}

fn main() {
    // this isn't actually unsafe :3
    #[allow(static_mut_refs)]
    unsafe {
        for entry in WalkDir::new(std::env::current_dir().unwrap()) {
            ALL_FILES.push(entry.unwrap().file_name().to_str().unwrap().to_string());
        }
    }

    let mut siv = cursive::default();
    siv.set_theme(Theme::terminal_default());

    let cb_sink = siv.cb_sink().clone();

    let (sx, rx) = crossbeam_channel::unbounded();

    siv.add_layer(
        LinearLayout::vertical()
            .child(FilesView::new(rx).full_height())
            .child(
                Dialog::new().title("Search files").content(
                    EditView::new()
                        .on_submit(|s, _| s.quit())
                        .on_edit(move |_s, text, _| {
                            let text_clone = text.to_string();
                            let sender_clone = sx.clone();
                            let cb_sink_clone = cb_sink.clone();

                            // TODO: this should be a single thread that's created on app start once and then communicated with on each on_edit invocation here
                            thread::spawn(move || {
                                let filtered_files = get_filtered_files(&text_clone);

                                sender_clone.send(filtered_files).unwrap();
                                cb_sink_clone.send(Box::new(Cursive::noop)).unwrap();
                            });

                            // rayon::scope(|s| {
                            //     s.spawn(|_s| {
                            //         let filtered_files = get_filtered_files(&text_clone);

                            //         sender_clone.send(filtered_files).unwrap();
                            //         cb_sink.send(Box::new(Cursive::noop)).unwrap();
                            //     });
                            // });
                        })
                        .with_name("query_edit_view")
                        .full_width(),
                ),
            )
            .full_height(),
    );

    siv.run();
}

struct FilesView {
    files: Vec<&'static String>,
    rx: Receiver<Vec<&'static String>>,
}

impl FilesView {
    fn new(rx: Receiver<Vec<&'static String>>) -> Self {
        Self {
            files: get_filtered_files(""),
            rx,
        }
    }

    fn update(&mut self) {
        while let Ok(files) = self.rx.try_recv() {
            self.files = files;
        }
    }
}

impl View for FilesView {
    fn layout(&mut self, _: Vec2) {
        // Before drawing, we'll want to update the buffer
        self.update();
    }

    fn draw(&self, printer: &Printer) {
        // Print the end of the buffer
        for (i, line) in self.files.iter().rev().take(printer.size.y).enumerate() {
            printer.print((0, printer.size.y - 1 - i), line);
        }
    }
}

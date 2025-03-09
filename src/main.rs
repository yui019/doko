use cursive::{
    theme::Theme,
    view::{Nameable, Resizable},
    views::{Dialog, EditView, LinearLayout, TextView},
};
use doko::levenshtein;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use walkdir::WalkDir;

fn get_filtered_files<'a>(files: &'a Vec<String>, query: &str) -> Vec<&'a String> {
    let filtered_files: Vec<&String> = files
        .par_iter()
        .filter(|&file| levenshtein(&file, &query) < 2)
        .collect();

    return filtered_files;
}

fn set_linear_layout_children(linear_layout: &mut LinearLayout, files: Vec<&String>) {
    linear_layout.clear();

    for file in &files[..20] {
        linear_layout.add_child(TextView::new(*file));
    }
}

fn main() {
    let mut files: Vec<String> = vec![];

    for entry in WalkDir::new("/home/haris/") {
        files.push(entry.unwrap().file_name().to_str().unwrap().to_string());
    }

    let mut siv = cursive::default();
    siv.set_theme(Theme::terminal_default());

    // TODO: use this to filter files outside the main thread!! (right now it's a line that does nothing lolz)
    siv.cb_sink();

    siv.add_layer(
        LinearLayout::vertical()
            .child(
                LinearLayout::vertical()
                    .with_name("files_linear_layout")
                    .full_height(),
            )
            .child(
                Dialog::new().title("Search files").content(
                    EditView::new()
                        .on_submit(|s, _| s.quit())
                        .on_edit(move |s, text, _| {
                            let filtered_files = get_filtered_files(&files, text);

                            s.call_on_name(
                                "files_linear_layout",
                                move |linear_layout: &mut LinearLayout| {
                                    set_linear_layout_children(linear_layout, filtered_files)
                                },
                            );
                        })
                        .with_name("query_edit_view")
                        .full_width(),
                ),
            )
            .full_height(),
    );

    siv.run();
}

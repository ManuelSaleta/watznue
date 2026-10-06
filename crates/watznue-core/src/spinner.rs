use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

use crate::PendingPackage;

/*
Simple loading spinner
*/
pub fn start_loading_spinner(pending:Vec<PendingPackage>){
    let mut index = 0;
    //TODO: make a cuter spinner
    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    while pending.is_empty() {
        let spinner = frames[index % frames.len()];
        print!("\rFetching {spinner}");
        let _ = io::stdout().flush();
        index += 1;
        sleep(Duration::from_millis(100));

        if !pending.is_empty() {
            break;
        }
    }
}

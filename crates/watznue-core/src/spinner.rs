use std::io::{self, Write};
use std::thread::sleep;
use std::time::Duration;

/*
Simple loading spinner
*/
pub fn start_loading_spinner<F>(mut is_fetching_updates: F)
where
    F: FnMut() -> bool,
{
    let frames = [
        r"  `~[^-^]~' ",
        r" ,~-[^-^]-~'",
        r" ,--[^-^]--<",
        r"  ,-[^-^]-< ",
        r"  ,~[^-^]~< ",
        r"   /[^-^]/  ",
        r"   /[@_<]-< ",
        r"   /[^-^]~` ",
        r"  /-[^-^]-~`",
        r" >--[^-^]--'",
        r"  \-[^-^]-\ ",
        r"   \[^-^]\  ",
        r"  `~[^-^]-\ ",
        r"  `~[^-^]-~ ",
    ];
    let mut index = 0;

    while is_fetching_updates() {
        let spinner = frames[index % frames.len()];
        print!("\rFetching {spinner} ");
        let _ = io::stdout().flush();
        index += 1;
        sleep(Duration::from_millis(80));
    }

    print!("\r\x1B[2K");
    let _ = io::stdout().flush();
}


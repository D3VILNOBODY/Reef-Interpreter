pub struct Scanner<'a> {
    text: &'a str,
    cur: usize,
    dbg: u8,
}

impl<'a> Scanner<'a> {
    pub fn new(text: &'a str) -> Scanner {
        Scanner {
            text,
            cur: 0,
            dbg: 0,
        }
    }

    pub fn next_token() {

    }
}

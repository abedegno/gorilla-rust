use super::{Qb, Result};

impl Qb {
    /// Block until any key is pressed and return it. This is the
    /// `DO WHILE Char$ = "": Char$ = INKEY$: LOOP` pattern in the listing.
    pub fn wait_key(&mut self) -> Result<char> {
        loop {
            if let Some(c) = self.inkey()? {
                return Ok(c);
            }
            self.rest(0.01)?;
        }
    }

    /// LINE INPUT prompt; target$
    ///
    /// Echoes the prompt and what has been typed so far at the current
    /// cursor position, and returns when Enter is pressed.
    ///
    /// Do NOT add a `clear_keys()` here. `LINE INPUT` does not clear the
    /// keyboard buffer, and the listing clears it in exactly two places,
    /// neither of them this one: `GetNum#` and `SparklePause`. Adding it
    /// makes every test that queues input before calling this hang forever,
    /// which nothing catches except an external timeout.
    pub fn line_input(&mut self, prompt: &str) -> Result<String> {
        let (row, col) = (self.text.row, self.text.col);
        let mut buf = String::new();
        loop {
            self.locate(row, col);
            self.print(&format!("{prompt}{buf}_ "));
            match self.inkey()? {
                Some('\r') => break,
                Some('\u{8}') => {
                    buf.pop();
                }
                Some(c) if !c.is_control() => buf.push(c),
                _ => {}
            }
            self.rest(0.01)?;
        }
        self.locate(row, col);
        self.print(&format!("{prompt}{buf}  "));
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use crate::qb::Qb;

    #[test]
    fn line_input_collects_until_enter() {
        let mut q = Qb::headless(640, 350);
        for c in "Jon\r".chars() {
            q.push_key(c);
        }
        assert_eq!(q.line_input("Name: ").unwrap(), "Jon");
    }

    #[test]
    fn line_input_handles_backspace() {
        let mut q = Qb::headless(640, 350);
        for c in "Jonx\u{8}\r".chars() {
            q.push_key(c);
        }
        assert_eq!(q.line_input("Name: ").unwrap(), "Jon");
    }

    #[test]
    fn backspace_on_an_empty_line_does_nothing() {
        let mut q = Qb::headless(640, 350);
        for c in "\u{8}\u{8}A\r".chars() {
            q.push_key(c);
        }
        assert_eq!(q.line_input("").unwrap(), "A");
    }

    #[test]
    fn wait_key_returns_the_first_key_pressed() {
        let mut q = Qb::headless(640, 350);
        q.push_key('V');
        assert_eq!(q.wait_key().unwrap(), 'V');
    }

    #[test]
    fn clear_keys_empties_the_queue() {
        let mut q = Qb::headless(640, 350);
        q.push_key('a');
        q.push_key('b');
        q.clear_keys();
        assert_eq!(q.inkey().unwrap(), None);
    }
}

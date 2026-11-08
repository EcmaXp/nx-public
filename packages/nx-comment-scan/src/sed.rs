pub(crate) struct Invocation {
    pub(crate) inplace: bool,
    pub(crate) scripts: Vec<String>,
    pub(crate) files: Vec<String>,
}

pub(crate) fn invocation(args: &[String]) -> Invocation {
    let mut inplace = false;
    let mut scripts = Vec::new();
    let mut operands = Vec::new();
    let mut script_given = false;
    let mut options_done = false;
    let mut rest = args.iter();
    while let Some(arg) = rest.next() {
        if options_done || arg == "-" || !arg.starts_with('-') {
            operands.push(arg.clone());
            continue;
        }
        if arg == "--" {
            options_done = true;
            continue;
        }
        if let Some(long) = arg.strip_prefix("--") {
            let (name, value) = match long.split_once('=') {
                Some((name, value)) => (name, Some(value.to_string())),
                None => (long, None),
            };
            match name {
                "in-place" => inplace = true,
                "expression" => {
                    scripts.extend(value.or_else(|| rest.next().cloned()));
                    script_given = true;
                }
                "file" => {
                    if value.is_none() {
                        rest.next();
                    }
                    script_given = true;
                }
                "line-length" if value.is_none() => {
                    rest.next();
                }
                _ => {}
            }
            continue;
        }
        let cluster = &arg[1..];
        for (index, flag) in cluster.char_indices() {
            let attached = &cluster[index + flag.len_utf8()..];
            let mut value = || {
                if attached.is_empty() {
                    rest.next().cloned()
                } else {
                    Some(attached.to_string())
                }
            };
            match flag {
                'i' => {
                    inplace = true;
                    break;
                }
                'e' => {
                    scripts.extend(value());
                    script_given = true;
                    break;
                }
                'f' => {
                    value();
                    script_given = true;
                    break;
                }
                'l' => {
                    value();
                    break;
                }
                _ => {}
            }
        }
    }
    if !script_given && !operands.is_empty() {
        scripts.push(operands.remove(0));
    }
    Invocation {
        inplace,
        scripts,
        files: operands,
    }
}

pub(crate) fn inserted_lines(scripts: &[String]) -> Option<Vec<String>> {
    let script: Vec<char> = scripts.join("\n").chars().collect();
    let mut parser = Parser {
        chars: &script,
        at: 0,
        lines: Vec::new(),
    };
    parser.script()?;
    Some(parser.lines)
}

struct Parser<'a> {
    chars: &'a [char],
    at: usize,
    lines: Vec<String>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<char> {
        self.chars.get(self.at).copied()
    }

    fn next(&mut self) -> Option<char> {
        let found = self.peek()?;
        self.at += 1;
        Some(found)
    }

    fn skip_blanks(&mut self) {
        while self.peek().is_some_and(|c| c == ' ' || c == '\t') {
            self.at += 1;
        }
    }

    fn skip_separators(&mut self) {
        while self.peek().is_some_and(|c| c.is_whitespace() || c == ';') {
            self.at += 1;
        }
    }

    fn skip_line(&mut self) {
        while self.next().is_some_and(|c| c != '\n') {}
    }

    fn skip_word(&mut self) {
        while self.peek().is_some_and(|c| c != ';' && c != '\n') {
            self.at += 1;
        }
    }

    fn script(&mut self) -> Option<()> {
        loop {
            self.skip_separators();
            if self.peek().is_none() {
                return Some(());
            }
            self.address()?;
            self.skip_blanks();
            while self.peek() == Some('!') {
                self.at += 1;
                self.skip_blanks();
            }
            self.command()?;
        }
    }

    fn address(&mut self) -> Option<()> {
        if !self.point()? {
            return Some(());
        }
        self.skip_blanks();
        if self.peek() == Some(',') {
            self.at += 1;
            self.skip_blanks();
            if matches!(self.peek(), Some('+' | '~')) {
                self.at += 1;
            }
            if !self.point()? {
                return None;
            }
        }
        Some(())
    }

    fn point(&mut self) -> Option<bool> {
        match self.peek() {
            Some(c) if c.is_ascii_digit() => {
                while self.peek().is_some_and(|c| c.is_ascii_digit() || c == '~') {
                    self.at += 1;
                }
            }
            Some('$') => self.at += 1,
            Some('/') => {
                self.at += 1;
                self.delimited('/')?;
                self.regex_flags();
            }
            Some('\\') => {
                self.at += 1;
                let delimiter = self.next()?;
                self.delimited(delimiter)?;
                self.regex_flags();
            }
            _ => return Some(false),
        }
        Some(true)
    }

    fn regex_flags(&mut self) {
        while matches!(self.peek(), Some('I' | 'M')) {
            self.at += 1;
        }
    }

    fn command(&mut self) -> Option<()> {
        match self.next()? {
            'a' | 'i' | 'c' => self.text(),
            's' => self.substitute()?,
            'y' => {
                let delimiter = self.next()?;
                self.delimited(delimiter)?;
                self.delimited(delimiter)?;
            }
            '#' | 'r' | 'R' | 'w' | 'W' | 'e' => self.skip_line(),
            ':' | 'b' | 't' | 'T' | 'q' | 'Q' | 'l' | 'L' | 'v' => self.skip_word(),
            '{' | '}' | '=' | 'd' | 'D' | 'F' | 'g' | 'G' | 'h' | 'H' | 'n' | 'N' | 'p' | 'P'
            | 'x' | 'z' => {}
            _ => return None,
        }
        Some(())
    }

    fn text(&mut self) {
        self.skip_blanks();
        if self.peek() == Some('\\') {
            self.at += 1;
            if self.peek() == Some('\n') {
                self.at += 1;
            }
        }
        let mut line = String::new();
        while let Some(c) = self.next() {
            match c {
                '\n' => break,
                '\\' => match self.next() {
                    Some('\n' | 'n') => self.lines.push(std::mem::take(&mut line)),
                    Some(escaped) => line.push(escaped),
                    None => {}
                },
                _ => line.push(c),
            }
        }
        self.lines.push(line);
    }

    fn substitute(&mut self) -> Option<()> {
        let delimiter = self.next()?;
        self.delimited(delimiter)?;
        let replacement = self.delimited(delimiter)?;
        self.lines
            .extend(replacement.split('\n').map(str::to_string));
        while let Some(c) = self.peek() {
            match c {
                'w' => {
                    self.skip_line();
                    break;
                }
                'g' | 'p' | 'i' | 'I' | 'm' | 'M' | 'e' => self.at += 1,
                _ if c.is_ascii_digit() => self.at += 1,
                _ => break,
            }
        }
        Some(())
    }

    fn delimited(&mut self, delimiter: char) -> Option<String> {
        if delimiter == '\n' || delimiter == '\\' {
            return None;
        }
        let mut out = String::new();
        loop {
            match self.next()? {
                c if c == delimiter => return Some(out),
                '\\' => match self.next()? {
                    c if c == delimiter => out.push(c),
                    'n' | '\n' => out.push('\n'),
                    c => {
                        out.push('\\');
                        out.push(c);
                    }
                },
                c => out.push(c),
            }
        }
    }
}

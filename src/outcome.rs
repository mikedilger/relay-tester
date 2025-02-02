use colorful::{Color, Colorful};

#[derive(Debug, Default, Clone)]
pub struct Outcome {
    pub pass: Option<bool>,
    pub info: Option<String>,
    pub subs: Vec<usize>,
}

impl Outcome {
    pub fn pass(info: Option<String>) -> Outcome {
        Outcome {
            pass: Some(true),
            info,
            subs: Vec::new(),
        }
    }

    pub fn fail(info: Option<String>) -> Outcome {
        Outcome {
            pass: Some(false),
            info,
            subs: Vec::new(),
        }
    }

    pub fn err(info: String) -> Outcome {
        Outcome {
            pass: None,
            info: Some(info),
            subs: Vec::new(),
        }
    }

    fn subs_str(&self) -> String {
        if self.subs.is_empty() {
            String::new()
        } else {
            let mut s: String = "[".to_owned();
            let mut first: bool = true;
            for i in &self.subs {
                if !first {
                    s.push(' ');
                };
                first = false;
                s.push_str(&format!("sub{}", i));
            }
            s.push_str("]: ");
            s
        }
    }
}

impl Outcome {
    pub fn display(&self, required: bool) -> String {
        match required {
            false => match self.pass {
                None => match self.info {
                    None => format!("{}", "UNTESTED".color(Color::Grey50)),
                    Some(ref s) => format!("{} ({})", "UNTESTED".color(Color::Grey50), s),
                },
                Some(false) => match self.info {
                    None => format!("{}{}", self.subs_str(), "NO".color(Color::DarkGoldenrod)),
                    Some(ref s) => format!(
                        "{}{} ({})",
                        self.subs_str(),
                        "NO".color(Color::DarkGoldenrod),
                        s
                    ),
                },
                Some(true) => match self.info {
                    None => format!("{}{} ", self.subs_str(), "YES".color(Color::Green)),
                    Some(ref s) => {
                        format!("{}{} ({})", self.subs_str(), "YES".color(Color::Green), s)
                    }
                },
            },
            true => match self.pass {
                None => match self.info {
                    None => format!("{}", "UNTESTED".color(Color::Grey50)),
                    Some(ref s) => format!("{} ({})", "UNTESTED".color(Color::Grey50), s),
                },
                Some(false) => match self.info {
                    None => format!("{}{}", self.subs_str(), "FAIL".color(Color::Red3a)),
                    Some(ref s) => {
                        format!("{}{} ({})", self.subs_str(), "FAIL".color(Color::Red3a), s)
                    }
                },
                Some(true) => match self.info {
                    None => format!("{}{} ", self.subs_str(), "PASS".color(Color::Green)),
                    Some(ref s) => {
                        format!("{}{} ({})", self.subs_str(), "PASS".color(Color::Green), s)
                    }
                },
            },
        }
    }
}

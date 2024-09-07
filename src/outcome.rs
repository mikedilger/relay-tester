use colorful::{Color, Colorful};

#[derive(Debug, Default, Clone)]
pub struct Outcome {
    pub pass: Option<bool>,
    pub info: Option<String>,
}

impl Outcome {
    pub fn pass(info: Option<String>) -> Outcome {
        Outcome {
            pass: Some(true),
            info,
        }
    }

    pub fn fail(info: Option<String>) -> Outcome {
        Outcome {
            pass: Some(false),
            info,
        }
    }

    pub fn err(info: String) -> Outcome {
        Outcome {
            pass: None,
            info: Some(info),
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
                    None => format!("{}", "NO".color(Color::DarkGoldenrod)),
                    Some(ref s) => format!("{} ({})", "NO".color(Color::DarkGoldenrod), s),
                },
                Some(true) => match self.info {
                    None => format!("{}", "YES".color(Color::Green)),
                    Some(ref s) => format!("{} ({})", "YES".color(Color::Green), s),
                },
            },
            true => match self.pass {
                None => match self.info {
                    None => format!("{}", "UNTESTED".color(Color::Grey50)),
                    Some(ref s) => format!("{} ({})", "UNTESTED".color(Color::Grey50), s),
                },
                Some(false) => match self.info {
                    None => format!("{}", "FAIL".color(Color::Red3a)),
                    Some(ref s) => format!("{} ({})", "FAIL".color(Color::Red3a), s),
                },
                Some(true) => match self.info {
                    None => format!("{}", "PASS".color(Color::Green)),
                    Some(ref s) => format!("{} ({})", "PASS".color(Color::Green), s),
                },
            },
        }
    }
}

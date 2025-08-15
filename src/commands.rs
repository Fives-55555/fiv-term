use std::io::stdin;

pub struct Commands {
    com: Action,
    args: Vec<Arg>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Arg {
    Inner((String, String)),
    Flag(String),
}

#[derive(PartialEq, Eq, Clone)]
pub enum Action {
    Help,
    Stop,
    Unknown(String),
}

pub const STOP: Action = Action::Stop;

trait CommmandUtils {
    fn read_com(&self) -> Commands {
        let mut s = String::new();
        println!("Please input a Command, if you need help please enter 'help'.");
        let _ = stdin().read_line(&mut s);
        let c = match Commands::from_string(s) {
            Ok(c) => c,
            Err(_) => panic!(),
        };
        c
    }
    fn read_com_nc(&self) -> Commands {
        let mut s = String::new();
        println!("Please input a Command, if you need help please enter 'help'.");
        let _ = stdin().read_line(&mut s).unwrap();
        let c = match Commands::from_string(s) {
            Ok(c) => c,
            Err(_) => {
                println!("Wrong Command");
                return self.read_com_nc();
            }
        };
        c
    }
}

impl Commands {
    pub fn get_arg_inner(&self, str: &str) -> Result<(String, String), ()> {
        let arg = &self.args;
        for i in 0..arg.len() {
            match &arg[i] {
                Arg::Inner(s) => {
                    if s.0 == str {
                        return Ok(s.clone());
                    }
                }
                Arg::Flag(_) => continue,
            }
        }
        return Err(());
    }
    pub fn args(&self) -> Vec<Arg> {
        self.args.clone()
    }
    pub fn action(&self) -> Action {
        self.com.clone()
    }
    pub fn from_string(mut src: String) -> Result<Commands, ()> {
        src = src
            .trim_matches(|ch| ch == ' ' || ch == '\n' || ch == '\r')
            .to_string();
        if src.is_empty() {
            return Err(());
        };
        let (com, rest) = match src.split_once(' ') {
            Some(r) => r,
            None => {
                let com = match src.as_str() {
                    "help" => Action::Help,
                    "stop" => Action::Stop,
                    _ => Action::Unknown(src.to_string()),
                };
                return Ok(Commands {
                    com: com,
                    args: Vec::new(),
                });
            }
        };
        let com = match com {
            "help" => Action::Help,
            "stop" => Action::Stop,
            _ => Action::Unknown(com.to_string()),
        };
        let mut index: usize;
        let mut rest: &str = rest;
        let mut strtemp: String = String::new();
        let mut arg: Vec<Arg> = Vec::new();
        while rest.len() != 0 {
            if &rest[0..2] == "--" {
                rest = &rest[2..];
                index = 0;
                loop {
                    if index < rest.len() && &rest[index..index + 1] == "=" {
                        strtemp.clear();
                        strtemp.push_str(&rest[0..index]);
                        if &rest[index + 1..index + 2] == "\"" {
                            rest = &rest[index + 2..];
                            match rest.split_once('\"') {
                                Some((inner, resta)) => {
                                    arg.push(Arg::Inner((strtemp.clone(), inner.to_string())));
                                    if resta.len() > 0 {
                                        rest = &resta[1..];
                                    } else {
                                        rest = resta;
                                    }
                                }
                                None => return Err(()),
                            }
                        } else {
                            rest = &rest[index + 1..];
                            match rest.split_once(' ') {
                                Some((inner, resta)) => {
                                    arg.push(Arg::Inner((strtemp.clone(), inner.to_string())));
                                    rest = resta;
                                }
                                None => {
                                    arg.push(Arg::Inner((strtemp.clone(), rest.to_string())));
                                    return Ok(Commands {
                                        com: com,
                                        args: arg,
                                    });
                                }
                            }
                        }
                        break;
                    } else if rest.len() == index {
                        arg.push(Arg::Flag(rest[0..index].to_string()));
                        return Ok(Commands {
                            com: com,
                            args: arg,
                        });
                    } else if &rest[index..index + 1] == " " {
                        arg.push(Arg::Flag(rest[0..index].to_string()));
                        rest = &rest[index + 1..];
                        break;
                    } else {
                        index += 1;
                    }
                }
            } else if &rest[0..1] == "-" {
                arg.push(Arg::Flag(rest[1..2].to_string()));
                if rest.len() < 3 {
                    return Ok(Commands {
                        com: com,
                        args: arg,
                    });
                } else {
                    rest = &rest[3..];
                }
            } else {
                return Err(());
            }
        }
        Ok(Commands {
            com: com,
            args: arg,
        })
    }
}

impl Arg {
    pub fn inner(&self) -> &(String, String) {
        match self {
            Arg::Inner(x) => x,
            Arg::Flag(_) => unreachable!(),
        }
    }
}

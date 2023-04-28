use clap::{Arg, Command};

#[derive(Debug)]
pub(crate) struct Args {
    pub host: String,
}

impl Args {
    pub fn parse() -> Self {
        let cmd = Command::new("server")
            .about("communication server")
            .arg(
                Arg::new("listen lost")
                    .short('l')
                    .long("listen")
                    .default_value("127.0.0.1:8233"),
            )
            .get_matches();

        let host = cmd.get_one::<String>("listen lost").cloned().unwrap();

        Args { host }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            host: Default::default(),
        }
    }
}

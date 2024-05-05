use core::fmt::Display;
use encore::prelude::*;

extern crate alloc;
use alloc::borrow::Cow;

#[derive(Clone)]
pub struct Error {
    program_name: &'static str,
    message: Cow<'static, str>,
}

impl Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "Error: {}", self.message)?;
        writeln!(f, "Usage: {} input -o output", self.program_name)?;

        Ok(())
    }
}

#[derive(Debug)]
pub struct Args {
    /// The executable to compress
    pub input: &'static str,
    /// Where to write the compressed executable on disk
    pub output: &'static str,
}

#[derive(Default)]
struct ArgsRaw {
    input: Option<&'static str>,
    output: Option<&'static str>,
}

impl Args {
    pub fn parse(env: &Env) -> Result<Self, Error> {
        let mut args = env.args.iter().copied();
        let mut raw: ArgsRaw = Default::default();

        // By convention, the first argument is the program's name
        let program_name = args.next().unwrap();

        let err = |message| Error {
            program_name,
            message,
        };

        while let Some(arg) = args.next() {
            if arg.starts_with('-') {
                Self::parse_flag(arg, &mut args, &mut raw, &err)?;
                continue;
            }

            if raw.input.is_some() {
                return Err(err("Multiple input files specified".into()));
            }
            else {
                raw.input = Some(arg);
            }
        }

        Ok(Args{
            input: raw.input.ok_or_else(|| err("Missing input".into()))?,
            output: raw.output.ok_or_else(|| err("Missing output".into()))?,
        })
    }

    fn parse_flag(
        flag: &'static str,
        args: &mut dyn Iterator<Item = &'static str>,
        raw: &mut ArgsRaw,
        err: &dyn Fn(Cow<'static, str>) -> Error,
     ) -> Result<(), Error> {
        match flag {
            "-o" | "--output" => {
                let output = args
                    .next()
                    .ok_or_else(|| err(format!("Missing output filename after {}", flag).into()))?;

                if raw.output.is_some() {
                    return Err(err("Multiple output files specified".into()));
                }
                else {
                    raw.output = Some(output);
                }

                Ok(())
            }
            x => Err(err(format!("Unknown flag {}", x).into())),
        }
    }
}
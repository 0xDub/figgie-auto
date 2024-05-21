use std::fs::OpenOptions;
use std::io::{self, Write};


#[derive(Debug, Clone)]
pub enum CL {
    Pink,
    Purple,
    Green,
    DullGreen,
    Blue,
    DullRed,
    Red,
    Orange,
    Teal,
    DullTeal,
    Dull,
    End,
}

impl CL {
    pub fn get(&self) -> &str {
        match self {
            CL::Pink => "\x1b[38;5;201m",
            CL::Purple => "\x1b[38;5;135m",
            CL::Green => "\x1b[38;5;46m",
            CL::DullGreen => "\x1b[38;5;29m",
            CL::Blue => "\x1b[38;5;27m",
            CL::DullRed => "\x1b[38;5;124m",
            CL::Red => "\x1b[38;5;196m",
            CL::Orange => "\x1b[38;5;208m",
            CL::Teal => "\x1b[38;5;14m",
            CL::DullTeal => "\x1b[38;5;153m",
            CL::Dull => "\x1b[38;5;8m",
            CL::End => "\x1b[37m",
        }
    }
}

impl ToString for CL {
    fn to_string(&self) -> String {
        match self {
            CL::Pink => "\x1b[38;5;201m".to_string(),
            CL::Purple => "\x1b[38;5;135m".to_string(),
            CL::Green => "\x1b[38;5;46m".to_string(),
            CL::DullGreen => "\x1b[38;5;29m".to_string(),
            CL::Blue => "\x1b[38;5;27m".to_string(),
            CL::DullRed => "\x1b[38;5;124m".to_string(),
            CL::Red => "\x1b[38;5;196m".to_string(),
            CL::Orange => "\x1b[38;5;208m".to_string(),
            CL::Teal => "\x1b[38;5;14m".to_string(),
            CL::DullTeal => "\x1b[38;5;153m".to_string(),
            CL::Dull => "\x1b[38;5;8m".to_string(),
            CL::End => "\x1b[37m".to_string(),
        }
    }
}

impl CL {
    pub fn print(&self, content: &str) {
        println!("{}{}{}", self.to_string(), content, CL::End.to_string())
    }

    pub fn print_literal(&self, content: String) {
        println!("{}{}{}", self.to_string(), content, CL::End.to_string())
    }

    pub fn str(&self, content: &str) -> String {
        format!("{}{}{}", self.to_string(), content, CL::End.to_string())
    }
}

// =-= FileHandler =-= //
pub struct FileHandler {
    file: std::fs::File,
}

impl FileHandler {
    pub fn new(file_path: &str) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open(file_path)?;
        Ok(Self { file })
    }

    pub fn write_line(&mut self, content: String) -> io::Result<()> {
        writeln!(self.file, "{}", content)
    }
}
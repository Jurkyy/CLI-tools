use std::fs::*;
use std::io::*;
use std::path::*;
use std::process;

const BUFFER_SIZE: usize = 16 * 1024;

struct Flags {
    line_no: isize,
    filename: bool,
    number: bool,
    newline: bool,
}

fn _help() {
    print!(concat!(
        "Usage: ",
        env!("CARGO_PKG_NAME"),
        " [OPTIONS]... [FILE]...\n\n",
        "OPTIONS:\n",
        "  -f, --filenames, --filename   display filenames\n",
        "  -n, --number                  number all output lines \n",
        "  -l, --newline                 prints a newline after each file \n",
        "  -?, --help                    display this help and exit\n"
    ));
}

fn _error(_path: &Path, _message: &Error) {
    let mut _text: String = _message.to_string();
    eprintln!("{}: {}: {}", env!("CARGO_PKG_NAME"), _path.display(), _text);
    std::io::stderr().flush().unwrap();
}

fn main() {
    let _args = (std::env::args()).skip(1);
    let mut _file_names = Vec::new();
    let mut _flags = Flags {
        line_no: 1,
        filename: false,
        number: false,
        newline: false,
    };
    for _arg in _args {
        if _arg.starts_with("-") && _arg.len() > 1 {
            if _arg == "-f" || _arg == "--filenames" || _arg == "--filename" {
                _flags.filename = true;
            } else if _arg == "-n" || _arg == "--number" {
                _flags.number = true;
            } else if _arg == "-?" || _arg == "--help" {
                _help();
                process::exit(0);
            } else if _arg == "-l" || _arg == "--newline" {
                _flags.newline = true
            } else {
                println!("unrecognized option -- '{}'", _arg);
                process::exit(1);
            }
        } else {
            _file_names.push(_arg)
        }
    }

    let mut _buffer = [0u8; BUFFER_SIZE];

    for _file_name in _file_names {
        let mut _bytes = 0;
        let mut _byte = [0];
        let mut _last = [10];
        let mut _count = 0;
        let _path = Path::new(&_file_name);
        match File::open(_path) {
            Ok(ref mut _file) => {
                while {
                    match _file.read(&mut _buffer) {
                        Ok(_bytes) => _count = _bytes,
                        Err(_err) => {
                            _error(_path, &_err);
                            _count = 0
                        }
                    }
                    _count
                } > 0
                {
                    if _flags.filename && _bytes == 0 {
                        eprintln!("{}", _path.display())
                    }
                    _bytes += _count;
                    for _index in 0.._count {
                        _byte[0] = _buffer[_index];
                        if _last[0] == 10 {
                            if _flags.number {
                                print!("{0:4}\t", _flags.line_no);
                            }
                            _flags.line_no += 1;
                        }
                        print!("{}", _byte[0] as char);
                        std::io::stdout().flush().unwrap();

                        _last = _byte;
                    }
                }
                if _flags.newline {
                    println!();
                }
            }
            Err(_message) => _error(&_path, &_message),
        }
    }
}

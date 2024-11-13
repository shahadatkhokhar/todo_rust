use colored::*;
use std::fs::OpenOptions;
use std::fs::{ self };
use std::io::prelude::Read;
use std::io::{ self, BufReader, BufWriter, Write };
use std::path::Path;
use std::{ env, process };

pub struct Todo {
    pub todo: Vec<String>,
    pub todo_path: String,
    pub todo_bak: String,
    pub no_backup: bool,
}

impl Todo {
    pub fn new() -> Result<Self, String> {
        let todo_path: String = match env::var("TODO_PATH") {
            Ok(t) => t,
            Err(_) => {
                let home = env::var("HOME").unwrap();

                // Look for a legacy TODO file path
                let legacy_todo = format!("{}/TODO", &home);
                match Path::new(&legacy_todo).exists() {
                    true => legacy_todo,
                    false => format!("{}/.todo", &home),
                }
            }
        };

        let todo_bak: String = match env::var("TODO_BAK_DIR") {
            Ok(t) => t,
            Err(_) => String::from("/tmp/todo.bak"),
        };

        let no_backup = env::var("TODO_NOBACKUP").is_ok();

        let todofile = OpenOptions::new()
            .write(true)
            .read(true)
            .create(true)
            .open(&todo_path)
            .expect("Couldn't open the todofile");

        // Creates a new buf reader
        let mut buf_reader = BufReader::new(&todofile);
        let mut contents = String::new();

        // Loads "contents" string with data
        buf_reader.read_to_string(&mut contents).unwrap();

        let todo = contents.lines().map(str::to_string).collect();

        Ok(Self {
            todo,
            todo_path,
            todo_bak,
            no_backup,
        })
    }

    pub fn list(&self) {
        let stdout = io::stdout();

        let mut writer = BufWriter::new(stdout);
        let mut data = String::new();

        for (number, task) in self.todo.iter().enumerate() {
            if task.len() > 4 {
                let number = (number + 1).to_string().bold();
                let symbol = &task[..4];
                let task = &task[4..];
                if symbol == "[*] " {
                    // DONE
                    // If the task is completed, then it prints it with a strikethrough
                    data = format!("{} {}\n", number, task.strikethrough());
                } else if symbol == "[ ] " {
                    // NOT DONE
                    // If the task is not completed yet, then it will print it as it is
                    data = format!("{} {}\n", number, task);
                }
                writer.write_all(data.as_bytes()).expect("Failed to write to stdout");
            }
        }
    }

    pub fn add(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo add takes at least 1 argument");
            process::exit(1);
        }
        let todofile = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todofile");

        let mut buffer = BufWriter::new(todofile);
        for arg in args {
            if arg.trim().is_empty() {
                continue;
            }

            // Appends a new task/s to the file
            let line = format!("[ ] {}\n", arg);
            buffer.write_all(line.as_bytes()).expect("unable to write data");
        }
    }
    pub fn remove(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo rm takes at least 1 argument");
            process::exit(1);
        }

        let todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open the todo file");

        let mut buffer = BufWriter::new(todofile);

        for (pos, line) in self.todo.iter().enumerate() {
            if args.contains(&(pos + 1).to_string()) {
                continue;
            }
            let line = format!("{}\n", line);
            buffer.write_all(line.as_bytes()).expect("unable to write data");
        }
    }
    fn remove_file(&self) {
        match fs::remove_file(&self.todo_path) {
            Ok(_) => {}
            Err(e) => { eprintln!("Error while clearing todo file: {}", e) }
        };
    }
    //
    pub fn reset(&self) {
        if !self.no_backup {
            match fs::copy(&self.todo_path, &self.todo_bak) {
                Ok(_) => self.remove_file(),
                Err(_) => { eprint!("Couldn't backup the todo file") }
            }
        } else {
            self.remove_file();
        }
    }
    pub fn restore(&self) {
        fs::copy(&self.todo_bak, &self.todo_path).expect("unable to restore the backup");
    }
    pub fn sort(&self) {
        let newtodo: String;

        let mut todo = String::new();
        let mut done = String::new();

        for line in self.todo.iter() {
            if line.len() > 5 {
                if &line[..4] == "[ ] " {
                    let line = format!("{}\n", line);
                    todo.push_str(&line);
                } else if &line[..4] == "[*] " {
                    let line = format!("{}\n", line);
                    done.push_str(&line);
                }
            }
        }
        newtodo = format!("{}{}", &todo, &done);

        let mut todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Couldn't open todofile");

        todofile.write_all(newtodo.as_bytes()).expect("Cannot save the todofile")
    }

    pub fn done(&self, args: &[String]) {
        if args.is_empty() {
            eprintln!("todo done takes at least 1 argument");
            process::exit(1);
        }
        let todofile = OpenOptions::new()
            .write(true)
            .open(&self.todo_path)
            .expect("Cannot open todofile");
        let mut buffer = BufWriter::new(todofile);
        for (pos, line) in self.todo.iter().enumerate() {
            if line.len() > 5 {
                if args.contains(&(pos + 1).to_string()) {
                    if &line[..4] == "[ ] " {
                        let line = format!("[*] {}\n", &line[4..]);
                        buffer.write_all(line.as_bytes()).expect("Cannot mark it as done");
                    } else if &line[..4] == "[*] " {
                        let line = format!("[*] {}\n", &args[1]);
                        buffer.write_all(line.as_bytes()).expect("unable to write data");
                    }
                } else if &line[..4] == "[ ] " || &line[..4] == "[*] " {
                    let line = format!("{}\n", &line);
                    buffer.write_all(line.as_bytes()).expect("unable to write data");
                }
            }
        }
    }
    pub fn edit(&self, args: &[String]) {
        if args.is_empty() || args.len() != 2 {
            eprintln!("todo edit takes exact 2 arguments");
            process::exit(1);
        }

        let todofile = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.todo_path)
            .expect("Cannot open todofile for edit");

        let mut buffer = BufWriter::new(todofile);

        for (pos, line) in self.todo.iter().enumerate() {
            if line.len() > 5 {
                if args[0].contains(&(pos + 1).to_string()) {
                    if &line[..4] == "[ ] " {
                        let line = format!("[ ] {}\n", args[1]);
                        buffer.write_all(line.as_bytes()).expect("cannot save the edited todo");
                    } else if &line[..4] == "[*] " {
                        let line = format!("[*] {}\n", args[1]);
                        buffer.write_all(line.as_bytes()).expect("cannot save the edited todo");
                    }
                } else if &line[..4] == "[ ] " || &line[..4] == "[*] " {
                    let line = format!("{}\n", line);
                    buffer.write_all(line.as_bytes()).expect("cannot save the edited todo");
                }
            }
        }
    }
}
const TODO_HELP: &str =
    "Usage: todo [COMMAND] [ARGUMENTS]
Todo is a super fast and simple tasks organizer written in rust
Example: todo list
Available commands:
- add [TASK/s]
    adds new task/s
    Example: todo add \"buy carrots\"
- edit [INDEX] [EDITED TASK/s]
    edits an existing task/s
    Example: todo edit 1 banana
- list
    lists all tasks
    Example: todo list
- done [INDEX]
    marks task as done
    Example: todo done 2 3 (marks second and third tasks as completed)
- rm [INDEX]
    removes a task
    Example: todo rm 4
- reset
    deletes all tasks
- restore 
    restore recent backup after reset
- sort
    sorts completed and uncompleted tasks
    Example: todo sort
- raw [todo/done]
    prints nothing but done/incompleted tasks in plain text, useful for scripting
    Example: todo raw done
";
pub fn help() {
    // For readability
    println!("{}", TODO_HELP);
}

pub struct ShellApp {
    pub process_name: &'static str,
}

pub const FILE_MANAGERS: &[ShellApp] = &[EXPLORER, DIRECTORY_OPUS, TOTAL_COMMANDER];

pub const EXPLORER: ShellApp = ShellApp {
    process_name: "explorer.exe",
};

pub const DIRECTORY_OPUS: ShellApp = ShellApp {
    process_name: "dopus.exe",
};

pub const TOTAL_COMMANDER: ShellApp = ShellApp {
    process_name: "Totalcmd64.exe",
};

pub const NOTEPAD: ShellApp = ShellApp {
    process_name: "Notepad.exe",
};

use core::time;
use crossterm::{
    cursor::{self, MoveTo},
    event::{self, Event::Key, KeyCode, KeyEvent, read},
    execute,
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    },
};

use std::{
    io::{self, Write, stdout},
    thread::sleep,
    time::Duration,
};

pub fn start() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, terminal::SetSize(256, 256))?;
    let terminal_size = terminal::size()?;
    let mut cursor_position = (terminal_size.0 / 2, terminal_size.1 / 2);
    execute!(stdout, cursor::MoveTo(cursor_position.0, cursor_position.1))?;
    loop {
        let key_pressed = read()?;
        if let Some(key) = key_pressed.as_key_event() {
            match key.code {
                KeyCode::Right => cursor_position.0 = cursor_position.0.saturating_add(1),
                KeyCode::Left => cursor_position.0 = cursor_position.0.saturating_sub(1),
                KeyCode::Up => cursor_position.1 = cursor_position.1.saturating_sub(1),
                KeyCode::Down => cursor_position.1 = cursor_position.1.saturating_add(1),
                _ => (print!("{:?},", key.code)),
            }
            let moved_curser = cursor::MoveTo(cursor_position.0, cursor_position.1);
            execute!(stdout, moved_curser);
        }
    }
    // lxecute!(stdout, EnterAlternateScreen,)?;
    let size = terminal::size()?;
    // for _ in 0..erminal::size()?;
    // println!("{:?}", size);

    sleep(Duration::from_secs(2));
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn end() -> io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

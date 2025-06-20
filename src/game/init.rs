use crossterm::{
    cursor::{self, MoveTo},
    event::{KeyCode, read},
    execute,
    terminal::{
        self, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
    },
};

use std::{
    io::{self, Stdout, Write, stdout},
    thread::sleep,
    time::Duration,
};

pub fn start() -> io::Result<Stdout> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, terminal::SetSize(5, 5))?;
    print_wall(&mut stdout)?;
    // let terminal_size = terminal::size()?;
    // let mut cursor_position = (terminal_size.0 / 2, terminal_size.1 / 2);
    // execute!(stdout, cursor::MoveTo(cursor_position.0, cursor_position.1))?;
    // loop {
    //     let key_pressed = read()?;
    //     if let Some(key) = key_pressed.as_key_event() {
    //         match key.code {
    //             KeyCode::Right => cursor_position.0 = cursor_position.0.saturating_add(1),
    //             KeyCode::Left => cursor_position.0 = cursor_position.0.saturating_sub(1),
    //             KeyCode::Up => cursor_position.1 = cursor_position.1.saturating_sub(1),
    //             KeyCode::Down => cursor_position.1 = cursor_position.1.saturating_add(1),
    //             _ => print!("{:?},", key.code),
    //         }
    //         let moved_curser = cursor::MoveTo(cursor_position.0, cursor_position.1);
    //         execute!(stdout, moved_curser)?;
    //     }
    // }
    // lxecute!(stdout, EnterAlternateScreen,)?;
    //let size = terminal::size()?;
    // for _ in 0..erminal::size()?;
    // println!("{:?}", size);

    // sleep(Duration::from_secs(2));
    // execute!(stdout, LeaveAlternateScreen)?;
    // disable_raw_mode()?;
    Ok(stdout)
}

pub fn end() -> io::Result<()> {
    let mut stdout = stdout();
    execute!(stdout, LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

pub fn print_wall(stdout: &mut Stdout) -> io::Result<()> {
    let terminal_size = size()?;
    execute!(stdout, MoveTo(0, 0)).unwrap();
    let horizen = "#".repeat(terminal_size.0 as usize);
    write!(stdout, "{horizen}").unwrap();
    (1..=terminal_size.1).for_each(|y| {
        execute!(stdout, MoveTo(0, y)).unwrap();
        write!(stdout, "#").unwrap();
        execute!(stdout, MoveTo(terminal_size.0, y)).unwrap();
        write!(stdout, "#").unwrap();
    });
    execute!(stdout, MoveTo(0, terminal_size.1))?;
    write!(stdout, "{horizen}")?;
    Ok(())
}

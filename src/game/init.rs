use crossterm::{
    cursor::{self, MoveTo},
    execute,
    terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode, size,
    },
};

use std::{
    io::{self, Stdout, Write, stdout},
    process::exit,
};

pub fn start(playground: &mut [[char; 256]; 256]) -> io::Result<Stdout> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, cursor::Hide)?;
    let len = playground.len();
    for i in 0..len {
        playground[0][i] = '|';
        playground[i][0] = '_';
        playground[len - 1][i] = '|';
        playground[i][len - 1] = '—';
    }

    //print_wall(&mut stdout)?;
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
    exit(0);
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

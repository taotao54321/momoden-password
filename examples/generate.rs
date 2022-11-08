//! cargo run --example=generate --release -- 'おに???'

use momoden_password::*;

fn main() {
    let pattern = std::env::args().nth(1).expect("Usage: generate <pattern>");
    let pattern: Vec<_> = pattern.chars().collect();
    assert!(matches!(
        pattern.len(),
        Password::MIN_LEN..=Password::MAX_LEN
    ));

    let mut solver = Solver::new(pattern);
    solver.dfs();

    println!();
    println!("count: {}", solver.count);
}

#[derive(Debug)]
struct Solver {
    pattern: Vec<char>,
    password: Vec<PasswordChar>,
    count: u64,
}

impl Solver {
    fn new(pattern: Vec<char>) -> Self {
        Self {
            pattern,
            password: Vec::with_capacity(Password::MAX_LEN),
            count: 0,
        }
    }

    fn dfs(&mut self) {
        let pos = self.password.len();

        // 全ての文字が決まったら有効かどうかチェックして戻る。
        if pos == self.pattern.len() {
            let password = Password::new(&self.password).unwrap();
            if password.is_valid() {
                self.count += 1;
                println!("{}", password.display());
            }
            return;
        }

        // 枝刈り: 2 文字目が無効なら直ちに却下。
        if pos == 2 && Password::is_invalid_second_char(*self.password.last().unwrap()) {
            return;
        }

        let c = self.pattern[pos];
        if c == '?' {
            for pc in PasswordChar::all() {
                self.password.push(pc);
                self.dfs();
                self.password.pop().unwrap();
            }
        } else {
            let pc = PasswordChar::from_char(c).unwrap();
            self.password.push(pc);
            self.dfs();
            self.password.pop().unwrap();
        }
    }
}

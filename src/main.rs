use rand::Rng;
use strum::{
  EnumIter, EnumString,
  VariantNames, EnumVariantNames,
  IntoEnumIterator, ToString as EnumToString,
};

// #[derive(Clone,Copy)]
struct Dice {
  sides: usize,
}
impl Dice {
  fn new(sides: usize) -> Self {
    Self { sides }
  }
  fn throw<RNG: Rng + ?Sized>(&self, rng: &mut RNG) -> usize {
    rng.gen_range(0, self.sides) + 1 // upper bound exclusive so [0..n[+1 it is => [1..n]
  }
  fn sides(&self) -> usize {
    self.sides
  }
}

#[derive(EnumString, EnumVariantNames, Eq, PartialEq)]
enum GameMode {
  REMOVE,
  TOGGLE,
}

#[derive(EnumString, EnumVariantNames)]
enum DiceMode {
  ALLORONE,  // total all or all individual
  SELECTION, // sum 1 2 and 3
}

#[derive(EnumString, EnumVariantNames, EnumIter, EnumToString, Eq, PartialEq, Copy, Clone)]
enum AllOrOne {
  TOTAL,
  INDIVIDUAL,
}

impl AllOrOne {
  fn is_worth(&self, game: &Clacker, throws: &[(usize, usize)]) -> bool {
    match self {
      AllOrOne::TOTAL => !game.cells[throws.iter().map(|&(side, _)| side).sum::<usize>()],
      AllOrOne::INDIVIDUAL => throws
          .iter()
          .any(|(side, _)| !game.cells[*side])
    }
  }
}

struct Clacker {
  dices: Vec<Dice>,
  cells: Vec<bool>,
  goal_cells: usize,
  toggled_cells: usize,
  mode: GameMode,
  dice_mode: DiceMode,
  throws: usize,
}

impl Clacker {
  fn new(dices: Vec<Dice>, mode: GameMode, dice_mode: DiceMode) -> Self {
    let len = dices.iter().map(Dice::sides).sum();
    let cells = vec![false; len + 1]; // cba to shift back
    Self {
      dices,
      cells,
      goal_cells: len,
      toggled_cells: 0,
      mode,
      dice_mode,
      throws: 0,
    }
  }
  fn throw<RNG: Rng + ?Sized>(&self, rng: &mut RNG) -> Vec<(usize, usize)> {
    self.dices
        .iter()
        .map(|dice| (dice.throw(rng), dice.sides()))
        .collect()
  }
  fn handle(&mut self, choices: Vec<usize>) -> bool {
    // is game finished
    //assert!(choices.len() <= self.dices.len(),"You can not have more choices than dices"); // at least, in classic rules
    choices.iter().for_each(|&n| {
      match self.mode {
        GameMode::REMOVE => {
          if !self.cells[n] {
            self.cells[n] = true;
            self.toggled_cells += 1;
          };
        }
        GameMode::TOGGLE => {
          if self.cells[n] {
            self.toggled_cells -= 1;
          } else {
            self.toggled_cells += 1;
          };
          self.cells[n] = !self.cells[n];
        }
      };
    });
    self.throws += 1;
    self.toggled_cells >= self.goal_cells
  }
  fn check_overlap(&self, throws: &[(usize, usize)]) -> bool {
    let max = throws.iter().map(|&(side, _)| side).sum();
    let min = throws.iter().map(|&(side, _)| side).min().unwrap();
    (min..=max).any(|n| !self.cells[n])
  }
}

fn get_input() -> String {
  let mut input = String::new();
  std::io::stdin().read_line(&mut input).unwrap(); // unhandled result duh
  input.trim().to_string()
}

use std::io::Write;
use std::str::FromStr;

fn next<T: FromStr>() -> Option<T> {
  FromStr::from_str(&get_input()).ok()
}

fn prompt_loop<T: FromStr>(prompt: &str) -> T {
  loop {
    print!("{} : ", prompt);
    std::io::stdout().flush().unwrap(); // println does it but print doesn't so
    match next::<T>() {
      Some(t) => break t,
      None => print!("Invalid input, Retry : "), // i could recurse kek
    };
  }
}

const SHADE: char = '▚';
fn main() {
  let mut rng = rand::thread_rng();
  let dices = loop {
    let dices = prompt_loop::<usize>("Number of dices ?");
    if dices < 1 {
      println!("You can't play with no dices");
    } else {
      break dices;
    }
  };
  let dices: Vec<Dice> = (1..=dices)
      .map(|i| {
        let sides = loop {
          let sides = prompt_loop::<usize>(&format!("Number of sides for dice {}?", i));
          if sides < 2 {
            // i won't allow a spherical dice
            println!("You can't have a sideless dice");
          } else {
            break sides;
          }
        };
        Dice::new(sides)
      })
      .collect();
  let mode = prompt_loop::<GameMode>(&format!(
    "Choose one game mode : {}",
    GameMode::VARIANTS.join(" | ")
  ));
  let dice_mode = prompt_loop::<DiceMode>(&format!(
    "Choose the dice mode : {}",
    DiceMode::VARIANTS.join(" | ")
  ));

  let mut game = Clacker::new(dices, mode, dice_mode); // only mut for handle
  loop {
    println!(
      "The board state : {}",
      game.cells
          .iter()
          .enumerate()
          .skip(1)
          .map(|(i, &b)| if b { SHADE.to_string() } else { i.to_string() })
          .collect::<Vec<String>>()
          .join(" ")
    );
    let throws = game.throw(&mut rng);
    println!(
      "You threw the dice(s) : {}",
      throws
          .iter()
          .map(|&(side, sides)| format!("{}d{}", side, sides))
          .collect::<Vec<String>>()
          .join(" | ")
    );
    let choices = if throws.len() > 1 {
      match game.dice_mode {
        DiceMode::ALLORONE => {
          let possible_choices_iter = AllOrOne::iter();
          let possible_choices: Vec<AllOrOne> = if game.mode == GameMode::REMOVE {
            possible_choices_iter
                .filter(|s| s.is_worth(&game, &throws))
                .collect()
          } else {
            possible_choices_iter.collect()
          };
          if possible_choices.is_empty() {
            let choice = if possible_choices.len() > 1 {
              loop {
                let choice = prompt_loop::<AllOrOne>(&format!(
                  "Choose : {}",
                  possible_choices
                      .iter()
                      .map(|v| v.to_string())
                      .collect::<Vec<String>>()
                      .join(" | ")
                ));
                if possible_choices.contains(&choice) {
                  break choice;
                } else {
                  println!("This is not one of the possible choices")
                };
              }
            } else {
              let choice = possible_choices[0]; // Copy
              println!("Only {} is possible", choice.to_string());
              choice
            };
            match choice {
              AllOrOne::TOTAL => {
                let total = throws.iter().map(|&(side, _)| side).sum();
                println!("You chose to use the total, this gives : {}", total);
                vec![total]
              }
              AllOrOne::INDIVIDUAL => {
                println!("You chose to use all the individual values");
                throws.iter().map(|&(side, _)| side).collect()
              }
            }
          } else {
            println!("There are no possible moves to remove new cells, skipped..");
            Vec::new()
          }
        }
        DiceMode::SELECTION => {
          if game.mode == GameMode::REMOVE && game.check_overlap(&throws) {
            // if there are possibilities for the player // else autoskip
            let mut stack: Vec<usize> = Vec::new();
            println!("Current Stack : []"); // its empty we can cheat a little
            throws.iter().enumerate().for_each(|(nth, &(side, dice))| {
              let index = loop {
                let i = prompt_loop::<usize>(&format!(
                  "In which slot of the stack to put the dice {} ({}d{}) ?",
                  nth + 1,
                  side,
                  dice
                )) - 1; // natural | integer
                if i > stack.len() {
                  println!(
                    "The slot ({}) can not be out of bounds : [1;{}]",
                    i + 1,
                    // 1, // 0 + 1
                    stack.len() + 1
                  );
                  continue;
                } else {
                  break i;
                }
              };
              if index < stack.len() {
                stack[index] += side
              } else {
                stack.push(side)
              }
              println!(
                "Current Stack : [{}]",
                stack
                    .iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
              );
            });
            stack
          } else {
            println!("There are no possible moves to remove new cells, thus this move is automatically skipped");
            Vec::new()
          }
        }
      }
    } else {
      vec![throws[0].0]
    };
    if game.handle(choices) {
      break;
    }
  }
  println!("You won in {} throw(s)", game.throws);
}

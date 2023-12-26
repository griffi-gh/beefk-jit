pub enum BfChar {
  Inc,
  Dec,
  Left,
  Right,
  Input,
  Output,
  LoopStart,
  LoopEnd,
}

pub enum BfOpGroup {
  Inc(isize),
  Move(isize),
  Input,
  Output,
  LoopStart,
  LoopEnd,
}

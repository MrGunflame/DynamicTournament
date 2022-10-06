use snowflaked::sync::Generator;

const INSTANCE: u16 = 0;

pub static TOURNAMENT: Generator = Generator::new_unchecked(INSTANCE);
pub static ENTRANT: Generator = Generator::new_unchecked(INSTANCE);
pub static BRACKET: Generator = Generator::new_unchecked(INSTANCE);
pub static ROLE: Generator = Generator::new_unchecked(INSTANCE);

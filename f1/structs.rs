use std::fmt::{Display, Formatter, Result};

type Milliliter = i32;
type Gram = i32;

// struct Point {
//     x: i32,
//     y: i32,
// }

enum CupSize {
    Small,
    Medium,
    Large,
}

enum Ingredience {
    CoffeeBeans(Gram),
    Milk(Milliliter),
    Sugar(Gram),
}

struct Cup(CupSize);

// struct CoffeeMaker {
//     intensity: i32,
//     type: 
//     size: CupSize,
// }

struct CupOfCoffee {
    name: String,
    ingrediences: Vec<Ingredience>,
}

trait CoffeeMaker {
    fn make_coffee(&self, size: CupSize) -> CupOfCoffee;
}

struct FlatWhite{
    coffee: CupOfCoffee,
}

impl FlatWhite {
    fn new(beans: Gram, milk: Milliliter, sugar: Gram) -> Self {
        FlatWhite {
            coffee: CupOfCoffee {
                name: "Flat White".to_string(),
                ingrediences: vec![
                    Ingredience::CoffeeBeans(beans),
                    Ingredience::Milk(milk),
                    Ingredience::Sugar(sugar),
                ],
            },
        }
    }
}

impl Default for FlatWhite {
    fn default() -> Self {
         FlatWhite {
            coffee: CupOfCoffee {
                name: "Flat White".to_string(),
                ingrediences: vec![
                    Ingredience::CoffeeBeans(0),
                    Ingredience::Milk(0),
                    Ingredience::Sugar(0),
                ],
            },
        }
    }
}

impl From<Cappuccino> for FlatWhite {
    fn from(value: Cappuccino) -> Self {
        FlatWhite {
            coffee: CupOfCoffee {
                name: "Flat White".to_string(),
                ingrediences: vec![
                    Ingredience::CoffeeBeans(0),
                    Ingredience::Milk(0),
                    Ingredience::Sugar(0),
                ],
            },
        }
    }
}

type ErrorMessage = String;

impl TryFrom<Cappuccino> for FlatWhite {
    type Error = ErrorMessage;

    fn try_from(value: Cappuccino) -> Result<Self, Self::Error> {
        Ok(FlatWhite {
            coffee: CupOfCoffee {
                name: "Flat White".to_string(),
                ingrediences: vec![
                    Ingredience::CoffeeBeans(0),
                    Ingredience::Milk(0),
                    Ingredience::Sugar(0),
                ],
            },
        })
    }
}

impl CoffeeMaker for FlatWhite {
    fn make_coffee(&self, size: CupSize) -> CupOfCoffee {
       CupOfCoffee{
            name: "Flat White".to_string(),
            ingrediences: vec![
                Ingredience::CoffeeBeans(20),
                Ingredience::Milk(150),
                Ingredience::Sugar(5),
            ]
       }
    }
}



struct Cappuccino {
    coffee: CupOfCoffee,
}

impl Default for Cappuccino {
    fn default() -> Self {
        Cappuccino {
            coffee: CupOfCoffee {
                name: "Cappuccino".to_string(),
                ingrediences: vec![
                    Ingredience::CoffeeBeans(0),
                    Ingredience::Milk(0),
                    Ingredience::Sugar(0),
                ],
            },
        }
    }
}

// enum DoJob {
//     Clean,
//     Cup(CupSize),
//     Coffee(Coffee),
// }

// impl Point {
//     fn new(x: i32, y: i32) -> Self {
//         Point { x, y }
//     }

//     fn sum(&self, z: i32) -> i32 {
//         if z < 0 {
//             return 0;
//         }

//         self.x + self.y + z
//     }
// }

// impl Display for Point {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result {
//         "({}, {})".fmt(f, self.x, self.y)
//     }
// }


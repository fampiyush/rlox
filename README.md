A tree walk lox interpreter written in Rust.\
Lox is a dynamically typed, interpreted scripting language designed by Bob Nystom for his book "Crafting Interpreters".

## How to run
- Clone the repository.
- Make sure rust is installed in your system.
- In the directory run ```cargo run example.lox```

## Lox Features
- Arithmetic operators (+, -, *, /)
- Comparison operators (<, >, <=, >=, ==)
- Logical operators (and, or, !)
- Variables
- Functions
- Conditional statements (if, if-else)
- Loops (for, while)
- Classes
- Inheritance

## Example
```
class Doughnut {
  cook() {
    print "Fry until golden brown.";
  }
}

class BostonCream < Doughnut {
  cook() {
    super.cook();
    print "Pipe full of custard and coat with chocolate.";
  }
}

fun cooking(num) {
    var dish = BostonCream();
    print num;
    dish.cook();
}

for(var i=0; i<5; i=i+1) {
    cooking(i);
}
```
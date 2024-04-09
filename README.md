# Sudoku Solver

A quick little project exercising Rust fundamentals.

While I was playing Sudoku one day, an intrusive thought entered my head: "I bet I could write a solver." This thought was surely constructed from my recent endeavors into the [Advent of Code](https://github.com/Foshkey/advent-of-code) (2023), particularly practicing Rust while solving those fun problems.

## How it works

For now, the code takes in `input.txt` as a grid of numbers, '.' is a blank space. For example:

```
...9..57.
..7...1.8
2......6.
...36...5
..1.824..
46...18..
.1......3
5.9...7..
..2..9...
```

This is inputted by `include_str!` which compiles the input into the program (not ideal if using the same executable for different inputs), but this was more an exercise than writing a program.

The solution is then outputted in the commandline.

## How it solves

1. Recursively go through each of the empty cells
2. Determines the possible numbers (looking at row, column, and house)
3. Loops through each of the possible numbers
4. Populates the cell with a possible number
5. Recursively go to the next empty cell
6. End state is determined by reaching the end of the board with possible numbers all the way

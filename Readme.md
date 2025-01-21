# roll

## architecture

### tokenizer

tokenizes the inputted expression.

### parser

parses the tokens into an AST basically.

### evaluator

evaluates the expression.

#### math expression

math is evaluated using common precedence logic.

- parens
- mul/div/mod
- add/sub

#### roll expression

roll expressions always starts with:

- {rolls}d{sides}
  - you can omit the {rolls}

thats all you need to roll.

you can also do this to roll dice:

- {rolls}d[{min}..{max}]
  - this rolls a die which has sides ranging from {min} to {max} (its inclusive)
- {rolls}d[{side1}, {side2}, {side3}]
  - this rolls a die with the specified values for its sides

a roll expression is built using a queue.
first the dice rolls are added to the queue and then all modifiers are enqueued
in order.

and expression `2d20k` produces the queue:

- **2d20**: roll 2d20
- **k**: keep highest roll
- sum the remaining rolls

##### modifiers

modifiers change the outcome of a roll in various ways such as adding or
removing rolls from the pool of existing rolls, or rerolling them.

- **(k|kh){integer}**: keep the {integer} highest rolls
- **kl{integer}**: keep the {integer} lowest rolls
- **(d|dl){integer}**: drop the {integer} lowest rolls
- **dh{integer}**: drop the {integer} highest rolls
- **!{integer}**: roll another die for each die in the pool which rolled its max \
value and keep doing so to the new dice being added until a maximum of {integer} dice have been added
  - logic can be altered with a condition
  - when a die explodes, the extra roll is added to the existing value and they
    are considered the same roll for modifiers such as `k`
- **r{integer}**: reroll each die in the pool which rolled its min value and \
keep doing so to the rerolled dice until the die has been rerolled a maximum of {integer} times
  - logic can be altered with a condition

###### conditions

conditions can be applied to some modifiers to change when they trigger

- **={value}**: trigger when equal to {value}
- **~={value}**: trigger when not equal to {value}
- **>{value}**: trigger when greater than {value}
- **<{value}**: trigger when less than {value}
- **>={value}**: trigger when greater or equal to {value}
- **<={value}**: trigger when less or equal to {value}

##### more examples

`7d6dl`:

- **7d6**: roll 7d6
- **dl**: drop lowest roll
- sum the remaining rolls

`3d6k2r=2!3`:

- **3d6**: roll 3d6
- **k2**: keep 2 highest rolls
- **r=2**: reroll results equal to 2
- **!3**: explode each die up to 3 times

`3d4d!2>=3r2!3k2`:

- **3d4**: roll 3d4
- **d**: drop the lowest roll
- **!2>=3**: explode up to 2 times if greater than or equal to 3
- **r2**: reroll results equal to one up to 2 times each
- **!3**: explode each die up to 3 times
- **k2**: keep 2 highest rolls in the pool

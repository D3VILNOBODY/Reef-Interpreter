# Reef

Reef is a programming language i am developing for one of my computer science
projects in college! Written wholly in rust, its a language based around
simplicity and efficiency. I hope to make it turing complete by the project
deadline, but if i complete it before then, ill hopefully add some more features.

Its mainly inspired by lua, c++, and rust, and similarities to each of these can
be found in the syntax.

# Running

Simply clone the repository and build using cargo. The reef-core and reef-syntax library should be compiled at the same time.

There are a few flags that can be passed to alter the behaviour of the program:
- `-f`: specify the file to read code from. If not passed, the program enters REPL mode
- `--debug`: enables debug features for parts of the interpreter if the value is greater than 0


# whenpathchanges

> Cross-platform daemon that will in a CPU efficient way spy on the kernel fs calls and if they modify a path u choose will in microseconds after <do something>

- when u use without a command arg will just print about the change to the terminal
`whenpathchanges file`

- notify urself about the change with rofi (have to have rofi installed)
`whenpathchanges dir rofi -e "{} was changed just now!"`

- can take intense measures and right away remove the changed file
`whenpathchanges dir rm {}`

- Good security usage is for example to run for ur .<shell>rc and other .<something>rc, that u run on either every user log in start or on every run of a new shell:

`whenpathchanges ~/.zshrc rofi -e "ur {} changed! if that was not u, act fast and don't start a new zsh shell!"`

- can run for multiple paths if provide a file with paths (each path has to be on a new line (will fail if one path does not exist unless --force))

`whenpathchanges --pathsfile pathsfile rofi -e "{} changed!"`


# whenpathchanges

### Cross-platform daemon that will in a CPU efficient way spy on the kernel fs calls and if they modify a path u choose will in microseconds after <do something>

Should file or any file/s in dir not be changed by some program without u knowing about it as quickly as possible? 
U will know in microsecond speed that it changed, better: 
    u can command what must happen in the moment of that change

Good security usage is for example to run for ur .<shell>rc and other .<something>rc, that u run on either every user log in start or on every run of a new shell:

```bash
whenpathchanges ~/.zshrc rofi -e "ur {} changed! if that was not u, act fast and don't start a new zsh shell!"
```

```bash

```

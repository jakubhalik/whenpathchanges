# whenpathchanges

### Cross-platform daemon that will in a CPU efficient way spy on the kernel fs calls and if they modify a path u choose will in microseconds after <do something>

Should file or any file/s in dir not be changed by some program without u knowing about it as quickly as possible? 
U will know in microsecond speed that it changed, better: 
    u can command what must happen in the moment of that change

- Good security usage is for example to run for ur .<shell>rc and other .<something>rc, that u run on either every user log in start or on every run of a new shell:

```bash
whenpathchanges ~/.zshrc rofi -e "ur {} changed! if that was not u, act fast and don't start a new zsh shell!"
```

- when u use without a command arg will just print about the change to the terminal

```bash
whenpathchanges file
```

- notify urself about the change with rofi (have to have rofi installed)

```bash
whenpathchanges dir rofi -e "{} was changed just now!"
```

- can take intense measures and right away remove the changed file

```bash
whenpathchanges dir rm {}
```

- can run for multiple paths if provide a file with paths (each path has to be on a new line (will fail if one path does not exist unless --force))

```bash
whenpathchanges --pathsfile pathsfile rofi -e "{} changed!"
```

U can make whenpathchanges run for a path/paths automatically on every boot, each whenpathchanges daemon will be on idle taking only about 3 MBs of RAM
If u running on linux with systemd u can:

```bash
# have to actually be in the path of the repo
source dev_utils.sh
tldrify_and_install whenpathchanges
su
source dev_utils.sh
systemdify '/usr/bin/whenpathchanges ~/.zshrc rofi -e "ur {} has been changed!"' when_path_zshrc 1 x
```


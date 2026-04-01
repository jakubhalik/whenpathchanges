tldrify_and_install () {
	randomzus=$RANDOM$RANDOM$RANDOM 
	echo '
    tldrify() {
      cp $1.md ~/.cache/tldr/pages/common/$1.md
      cp $1.md ~/.cache/tlrc/pages.en/common/$1.md
    }
    tldrify $1
    read -sp "Enter sudo pass: " sudo_pass 
    cargo build --release
    echo "$sudo_pass" | sudo -S install target/release/$1 /usr/bin/
  ' > /tmp/$randomzus
	chmod +x /tmp/$randomzus
	/tmp/$randomzus $1
	rm /tmp/$randomzus
}

make_service_run_on_boot() {
  default=5
  test "$3" && RestartSec="$3" || RestartSec="$default"
  echo "
[Unit]
Description=My Custom Boot Command
After=network.target

[Service]
Type=simple

# REQUIRED: Give the background service access to your screen so 'rofi' doesn't crash
Environment=DISPLAY=:0
Environment=WAYLAND_DISPLAY=wayland-1
Environment=XDG_RUNTIME_DIR=/run/user/1000

# REQUIRED: Wrap in bash so the '~' symbol is understood
ExecStart=/usr/bin/bash -c '$1'

User=$4
Restart=on-failure
RestartSec=$RestartSec

[Install]
WantedBy=multi-user.target
" > /etc/systemd/system/$2.service

  # Reload MUST happen before start!
  systemctl daemon-reload
  systemctl enable $2.service
  systemctl start $2.service
}

systemdify() {
  # REQUIRED: Quotes prevent bash from shattering your first argument
  make_service_run_on_boot "$@"
}

systemdify() {
  make_service_run_on_boot $@
}

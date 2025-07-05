#!/system/bin/sh
# 文件路径: /Thanatos/magisk/customize.sh

# This script is executed during the Magisk module installation

# 1. A's a root user, Magisk will extract the module files to $MODPATH
#    This script has access to utility functions defined by Magisk.
#    ui_print, set_perm, set_perm_recursive, etc.

ui_print "*******************************"
ui_print " Thanatos Core Service Installer "
ui_print "*******************************"

# 2. Set permissions for the daemon binary
ui_print "- Setting permissions for thanatosd..."
# Set executable permissions (rwxr-xr-x) for the daemon
# Owner: root (0), Group: root (0)
set_perm_recursive $MODPATH/system/bin 755 0 0

# 3. Handle SELinux policies
# Magisk v21+ automatically handles sepolicy.rule files.
# We just need to ensure the file exists.
if [ -f "$MODPATH/sepolicy.rule" ]; then
  ui_print "- SELinux policies will be applied on next reboot."
else
  ui_print "- No sepolicy.rule file found, skipping SELinux patching."
fi

ui_print "- Installation complete."
ui_print "- Please reboot your device to activate Thanatos."

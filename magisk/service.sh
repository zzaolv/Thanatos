#!/system/bin/sh
# 文件路径: /Thanatos/magisk/service.sh

# This script is executed by Magisk post-fs-data or late_start service mode.
# We use late_start service mode to ensure the system is mostly booted.

# Wait until the boot process is complete to avoid race conditions.
while [ "$(getprop sys.boot_completed)" != "1" ]; do
  sleep 2
done

# The module's installation directory
MODDIR=${0%/*}

# Log file for debugging
LOGFILE=$MODDIR/thanatos.log
# Database file path
DB_PATH=$MODDIR/thanatos.db
# Socket file path
SOCKET_PATH="/data/local/tmp/thanatosd.sock"
# The daemon binary
BINARY="$MODDIR/system/bin/thanatosd"

# --- Safety Break ---
# A simple way to disable the daemon without uninstalling the module.
# Just create a file named DISABLED in the module directory.
DISABLED_FLAG="$MODDIR/DISABLED"
if [ -f "$DISABLED_FLAG" ]; then
  echo "Thanatos is disabled by flag file. Exiting." > $LOGFILE
  exit 0
fi

# Clean up old log file
rm -f $LOGFILE

# Function to start the daemon
start_daemon() {
    # Clean up any leftover socket file from a previous crash
    rm -f $SOCKET_PATH
    
    echo "Starting thanatosd daemon..." >> $LOGFILE
    echo "Binary: $BINARY" >> $LOGFILE
    echo "Socket: $SOCKET_PATH" >> $LOGFILE
    echo "Log: $LOGFILE" >> $LOGFILE
    echo "Database: $DB_PATH" >> $LOGFILE
    
    # Start the daemon in a new session to detach it from this script's lifecycle.
    # Redirect all output (stdout and stderr) to the log file.
    nohup $BINARY > $LOGFILE 2>&1 &
    
    sleep 2
    
    # Verify that the process has started
    if pgrep -f "$BINARY" >/dev/null; then
        echo "Thanatosd started successfully." >> $LOGFILE
    else
        echo "Error: Failed to start thanatosd." >> $LOGFILE
    fi
}

# Start the daemon
start_daemon

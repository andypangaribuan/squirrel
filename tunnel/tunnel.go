/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package tunnel

import (
	"net"
	"os"
	"os/exec"
	"strconv"
	"strings"
	"syscall"
	"time"
)

func isPortOpen(port string) bool {
	conn, err := net.DialTimeout("tcp", "127.0.0.1:"+port, 200*time.Millisecond)
	if err != nil {
		return false
	}
	_ = conn.Close()
	return true
}

func getPidByPort(port string) int {
	cmd := exec.Command("lsof", "-ti", "tcp:"+port)
	output, _ := cmd.Output()
	pidStr := strings.TrimSpace(string(output))
	if pidStr == "" {
		return 0
	}
	lines := strings.Split(pidStr, "\n")
	for _, line := range lines {
		pid, _ := strconv.Atoi(line)
		if pid > 0 && pid != os.Getpid() {
			return pid
		}
	}
	return 0
}

func startWorker(name string) (*exec.Cmd, error) {
	exe, err := os.Executable()
	if err != nil {
		return nil, err
	}

	cmd := exec.Command(exe, "tunnel", "worker", name)
	// We don't setsid here because the watchdog will manage this process
	cmd.Stdin = nil
	cmd.Stdout = nil
	cmd.Stderr = nil

	if err := cmd.Start(); err != nil {
		return nil, err
	}

	return cmd, nil
}

func startWatchdog(name string) (int, error) {
	exe, err := os.Executable()
	if err != nil {
		return 0, err
	}

	cmd := exec.Command(exe, "tunnel", "watchdog", name)

	cmd.SysProcAttr = &syscall.SysProcAttr{
		Setsid: true,
	}

	cmd.Stdin = nil
	cmd.Stdout = nil
	cmd.Stderr = nil

	if err := cmd.Start(); err != nil {
		return 0, err
	}

	return cmd.Process.Pid, nil
}

func stopTunnel(pid int) error {
	if pid <= 0 {
		return nil
	}

	process, err := os.FindProcess(pid)
	if err != nil {
		return nil
	}

	err = process.Signal(syscall.Signal(0))
	if err != nil {
		return nil // Already dead
	}

	return process.Signal(syscall.SIGTERM)
}

func isTunnelRunning(pid int) bool {
	if pid <= 0 {
		return false
	}
	process, err := os.FindProcess(pid)
	if err != nil {
		return false
	}

	// Double check if the process is actually alive
	err = process.Signal(syscall.Signal(0))
	if err != nil {
		return false
	}

	// On macOS/Linux, verify this is actually our watchdog process
	// This prevents issues with PID reuse.
	cmd := exec.Command("ps", "-p", strconv.Itoa(pid), "-o", "command=")
	output, err := cmd.Output()
	if err != nil {
		return true // Fallback to PID check if ps fails
	}

	cmdLine := string(output)
	return strings.Contains(cmdLine, "tunnel watchdog")
}

func syncStatus(name string, isRunning bool, currentStatus string, localPort string) string {
	displayStatus := currentStatus

	// 1. If the watchdog is NOT running, but it was supposedly active.
	if !isRunning && currentStatus != "" {
		// If it's already "disconnected-ready", it means we've shown "disconnected" for at least one tick.
		// Now we try to restart the watchdog.
		if currentStatus == "disconnected-ready" {
			newPid, err := startWatchdog(name)
			if err == nil {
				cfg, _ := loadConfig()
				_ = cfg.atomicUpdate(name, func(t *stuTunnelConfig) {
					t.PID = newPid
					t.Status = "reconnecting"
				})
				return "reconnecting"
			}
			return "disconnected"
		}

		// If it's "disconnected", it means we just detected the crash.
		// Move to "disconnected-ready" which will be caught in the NEXT tick (approx 1s later).
		if currentStatus == "disconnected" {
			displayStatus = "disconnected-ready"
		} else {
			// First step of failure: show it as "disconnected"
			displayStatus = "disconnected"
		}
	} else if !isRunning {
		displayStatus = ""
	} else if displayStatus == "connected" {
		if !isPortOpen(localPort) {
			displayStatus = "disconnected"
		}
	}

	// 2. If the status in config is out of sync with reality, update the config
	if displayStatus != currentStatus {
		cfg, err := loadConfig()
		if err == nil {
			_ = cfg.atomicUpdate(name, func(t *stuTunnelConfig) {
				if !isRunning && (displayStatus == "" || displayStatus == "disconnected" || displayStatus == "disconnected-ready") {
					t.PID = 0
					t.SshPid = 0
				}
				t.Status = displayStatus
			})
		}
	}

	// For the UI, "disconnected-ready" still looks like "disconnected"
	if displayStatus == "disconnected-ready" {
		return "disconnected"
	}

	return displayStatus
}

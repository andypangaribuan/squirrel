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
	"io"
	"log"
	"os"
	"os/exec"
	"os/signal"
	"path/filepath"
	"sync"
	"syscall"
	"time"
)

func runWorker(name string) {
	updateStatus := func(status string, sshPid int) {
		for i := range 5 {
			cfg, err := loadConfig()
			if err == nil {
				_ = cfg.atomicUpdate(name, func(t *stuTunnelConfig) {
					t.Status = status
					t.SshPid = sshPid
				})
				return
			}

			log.Printf("Worker [%s]: Failed to load config for status update (attempt %d): %v", name, i+1, err)
			time.Sleep(100 * time.Millisecond)
		}

		log.Printf("Worker [%s]: Failed to update status after retries.", name)
	}

	home, _ := os.UserHomeDir()
	logPath := filepath.Join(home, ".config", "squirrel", "tunnel.log")
	_ = os.MkdirAll(filepath.Dir(logPath), 0755)
	f, _ := os.OpenFile(logPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if f != nil {
		defer func() {
			_ = f.Close()
		}()

		mw := io.MultiWriter(f, os.Stderr)
		log.SetOutput(mw)
	}

	cfg, err := loadConfig()
	if err != nil {
		log.Printf("Worker [%s]: Failed to load config: %v", name, err)
		os.Exit(1)
	}

	t, found := cfg.getTunnel(name)
	if !found {
		os.Exit(1)
	}

	updateStatus("reconnecting", os.Getpid())
	time.Sleep(1 * time.Second)

	tunnel, err := startSSHTunnelGo(&t)
	if err != nil {
		log.Printf("Worker [%s]: Failed to start tunnel: %v", name, err)
		updateStatus("disconnected", os.Getpid())
		os.Exit(1)
	}

	updateStatus("connected", os.Getpid())
	err = <-tunnel.Wait()
	if err != nil {
		updateStatus("reconnecting", os.Getpid())
		os.Exit(1)
	}

	os.Exit(0)
}

func runWatchdog(name string) {
	home, _ := os.UserHomeDir()
	logPath := filepath.Join(home, ".config", "squirrel", "tunnel.log")
	_ = os.MkdirAll(filepath.Dir(logPath), 0755)

	f, _ := os.OpenFile(logPath, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0644)
	if f != nil {
		defer func() {
			_ = f.Close()
		}()

		log.SetOutput(f)
	}

	log.Printf("--- Watchdog Supervisor started for [%s] (PID: %d) ---", name, os.Getpid())

	sigChan := make(chan os.Signal, 1)
	signal.Notify(sigChan, syscall.SIGTERM, syscall.SIGINT)

	cfg, _ := loadConfig()
	_ = cfg.atomicUpdate(name, func(t *stuTunnelConfig) {
		t.PID = os.Getpid()
	})

	var currentWorker *exec.Cmd
	var mu sync.Mutex

	go func() {
		<-sigChan
		log.Printf("Watchdog [%s]: Termination signal received.", name)
		mu.Lock()
		if currentWorker != nil && currentWorker.Process != nil {
			_ = currentWorker.Process.Signal(syscall.SIGTERM)
		}
		mu.Unlock()

		cfg, _ := loadConfig()
		_ = cfg.atomicUpdate(name, func(t *stuTunnelConfig) {
			t.PID = 0
			t.SshPid = 0
			t.Status = ""
		})
		os.Exit(0)
	}()

	for {
		cfg, err := loadConfig()
		if err != nil {
			time.Sleep(1 * time.Second)
			continue
		}

		t, found := cfg.getTunnel(name)
		if !found {
			log.Printf("Watchdog [%s]: Tunnel configuration not found, exiting.", name)
			return
		}

		if t.LocalPort != "" {
			portPid := getPidByPort(t.LocalPort)
			if portPid > 0 && portPid != os.Getpid() {
				log.Printf("Watchdog [%s]: Cleaning up port %s (PID: %d)", name, t.LocalPort, portPid)
				_ = stopTunnel(portPid)
				time.Sleep(500 * time.Millisecond)
			}
		}

		log.Printf("Watchdog [%s]: Starting worker process...", name)
		worker, err := startWorker(name)
		if err != nil {
			log.Printf("Watchdog [%s]: Failed to start worker: %v", name, err)
			time.Sleep(3 * time.Second)
			continue
		}

		mu.Lock()
		currentWorker = worker
		mu.Unlock()

		err = worker.Wait()
		log.Printf("Watchdog [%s]: Worker process exited: %v", name, err)

		cfg, _ = loadConfig()
		_ = cfg.atomicUpdate(name, func(t *stuTunnelConfig) {
			t.Status = "disconnected"
			t.SshPid = 0
		})

		time.Sleep(1 * time.Second)
	}
}

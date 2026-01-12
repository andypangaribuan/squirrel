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
	"fmt"
	"log"
	"os"
	"path/filepath"
	"time"

	"github.com/charmbracelet/bubbles/key"
	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/huh"
)

func interactiveMenu() {
	cfg, err := loadConfig()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	maxN, maxP := calculateMaxLengths(cfg.Tunnels)
	var items []list.Item
	for _, t := range cfg.Tunnels {
		items = append(items, tunnelItem{
			config:     t,
			running:    isTunnelRunning(t.PID),
			maxNameLen: maxN,
			maxPortLen: maxP,
		})
	}

	d := list.NewDefaultDelegate()
	d.ShowDescription = false
	d.SetHeight(1)
	d.SetSpacing(0)

	l := list.New(items, d, 0, 0)
	l.Title = "SSH Tunnels"
	l.SetShowTitle(true)
	l.SetShowStatusBar(false)
	l.SetShowHelp(false)
	l.SetFilteringEnabled(true)
	l.Filter = list.UnsortedFilter
	l.KeyMap.Quit.SetKeys("q", "ctrl+c")

	l.Styles.Title = titleStyle
	l.AdditionalFullHelpKeys = func() []key.Binding {
		return []key.Binding{addKey, stopAllKey}
	}
	l.AdditionalShortHelpKeys = func() []key.Binding {
		return []key.Binding{addKey, stopAllKey}
	}

	m := itemModel{
		list:  l,
		state: stateList,
	}

	p := tea.NewProgram(m, tea.WithAltScreen())
	if _, err := p.Run(); err != nil {
		fmt.Printf("Error running program: %v\n", err)
		os.Exit(1)
	}
}

func startTunnelLogic(name string) {
	cfg, _ := loadConfig()
	t, _ := cfg.getTunnel(name)
	if isTunnelRunning(t.PID) {
		return
	}
	pid, err := startWatchdog(name)
	if err == nil {
		t.PID = pid
		_ = cfg.updateTunnel(t)
	}
}

func stopTunnelLogic(name string) {
	cfg, _ := loadConfig()
	t, _ := cfg.getTunnel(name)

	// 1. Try to stop the known watchdog PID
	if t.PID > 0 {
		_ = stopTunnel(t.PID)
	}

	// 2. Also check if anything else is currently using the local port
	if t.LocalPort != "" {
		portPid := getPidByPort(t.LocalPort)
		if portPid > 0 && portPid != os.Getpid() {
			_ = stopTunnel(portPid)
		}
	}

	t.PID = 0
	t.SshPid = 0
	t.Status = ""
	_ = cfg.updateTunnel(t)
}

func stopAllTunnelsLogic() {
	cfg, _ := loadConfig()
	for _, t := range cfg.Tunnels {
		// Stop by PID
		if t.PID > 0 {
			_ = stopTunnel(t.PID)
		}

		// Stop by Port
		if t.LocalPort != "" {
			portPid := getPidByPort(t.LocalPort)
			if portPid > 0 && portPid != os.Getpid() {
				_ = stopTunnel(portPid)
			}
		}

		t.PID = 0
		t.SshPid = 0
		t.Status = ""
		_ = cfg.updateTunnel(t)
	}

	home, _ := os.UserHomeDir()
	logPath := filepath.Join(home, ".config", "squirrel", "tunnel.log")
	_ = os.WriteFile(logPath, []byte(""), 0644)
}

func deleteTunnelLogic(name string) {
	var confirm bool
	form := huh.NewForm(
		huh.NewGroup(
			huh.NewConfirm().
				Title(fmt.Sprintf("Are you sure you want to delete tunnel '%s'?", name)).
				Value(&confirm),
		),
	)
	if err := form.Run(); err != nil || !confirm {
		return
	}

	cfg, _ := loadConfig()
	if t, ok := cfg.getTunnel(name); ok {
		if t.PID > 0 {
			_ = stopTunnel(t.PID)
		}
		if t.LocalPort != "" {
			portPid := getPidByPort(t.LocalPort)
			if portPid > 0 && portPid != os.Getpid() {
				_ = stopTunnel(portPid)
			}
		}
	}
	_ = cfg.deleteTunnel(name)
	fmt.Printf("Tunnel '%s' deleted.\n", name)
	time.Sleep(500 * time.Millisecond)
}

func addTunnelLogic() {
	var name, host, pass, remote, local, identity, proxy string

	form := huh.NewForm(
		huh.NewGroup(
			huh.NewInput().Title("Unique name:").Value(&name),
			huh.NewInput().Title("SSH Host:").Value(&host),
			huh.NewInput().Title("SSH Password (optional):").EchoMode(huh.EchoModePassword).Value(&pass),
			huh.NewInput().Title("Identity File (optional):").Value(&identity),
			huh.NewInput().Title("Proxy Command (optional):").Value(&proxy),
			huh.NewInput().Title("Remote Address:Port:").Value(&remote),
			huh.NewInput().Title("Local Port:").Value(&local),
		),
	)

	if err := form.Run(); err != nil {
		return
	}

	if name == "" || host == "" || remote == "" || local == "" {
		fmt.Println("Error: Name, Host, Remote Address, and Local Port are required.")
		time.Sleep(2 * time.Second)
		return
	}

	cfg, _ := loadConfig()
	newTunnel := stuTunnelConfig{
		Name:         name,
		Host:         host,
		Password:     pass,
		IdentityFile: identity,
		ProxyCommand: proxy,
		RemoteAddr:   remote,
		LocalPort:    local,
	}

	err := cfg.addTunnel(newTunnel)
	if err != nil {
		fmt.Printf("Error adding tunnel: %v\n", err)
		time.Sleep(2 * time.Second)
		return
	}

	fmt.Printf("Tunnel '%s' added successfully.\n", name)
	time.Sleep(1 * time.Second)
}

func updateTunnelLogic(name string) {
	cfg, _ := loadConfig()
	t, _ := cfg.getTunnel(name)

	var currentName, host, pass, remote, local, identity, proxy string
	currentName = t.Name
	host = t.Host
	remote = t.RemoteAddr
	local = t.LocalPort
	identity = t.IdentityFile
	proxy = t.ProxyCommand

	form := huh.NewForm(
		huh.NewGroup(
			huh.NewInput().Title("Name:").Value(&currentName),
			huh.NewInput().Title("SSH Host:").Value(&host),
			huh.NewInput().Title("SSH Password:").EchoMode(huh.EchoModePassword).Value(&pass),
			huh.NewInput().Title("Identity File:").Value(&identity),
			huh.NewInput().Title("Proxy Command:").Value(&proxy),
			huh.NewInput().Title("Remote Address:Port:").Value(&remote),
			huh.NewInput().Title("Local Port:").Value(&local),
		),
	)

	if err := form.Run(); err != nil {
		return
	}

	if currentName != t.Name && currentName != "" {
		if isTunnelRunning(t.PID) {
			_ = stopTunnel(t.PID)
			t.PID = 0
		}
	}

	if currentName != "" {
		t.Name = currentName
	}
	if host != "" {
		t.Host = host
	}
	if pass != "" {
		t.Password = pass
	}
	t.IdentityFile = identity
	t.ProxyCommand = proxy
	if remote != "" {
		t.RemoteAddr = remote
	}
	if local != "" {
		t.LocalPort = local
	}

	err := cfg.renameTunnel(name, t)
	if err != nil {
		fmt.Printf("Error updating tunnel: %v\n", err)
	} else {
		fmt.Printf("Tunnel '%s' updated successfully.\n", t.Name)
	}
	time.Sleep(500 * time.Millisecond)
}

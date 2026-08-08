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
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"syscall"
)

func getConfigPath() string {
	home, _ := os.UserHomeDir()
	return filepath.Join(home, ".config", "squirrel", "tunnel.json")
}

func loadConfig() (*stuConfig, error) {
	path := getConfigPath()
	if _, err := os.Stat(path); os.IsNotExist(err) {
		return &stuConfig{Tunnels: []stuTunnelConfig{}}, nil
	}

	f, err := os.OpenFile(path, os.O_RDWR, 0600)
	if err != nil {
		return nil, err
	}
	defer func() {
		_ = f.Close()
	}()

	if err := syscall.Flock(int(f.Fd()), syscall.LOCK_EX); err != nil {
		return nil, err
	}
	defer func() {
		_ = syscall.Flock(int(f.Fd()), syscall.LOCK_UN)
	}()

	var cfg stuConfig
	if err := json.NewDecoder(f).Decode(&cfg); err != nil {
		return nil, err
	}

	return &cfg, nil
}

func saveConfig(cfg *stuConfig) error {
	path := getConfigPath()
	dir := filepath.Dir(path)
	if err := os.MkdirAll(dir, 0755); err != nil {
		return err
	}

	f, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE, 0600)
	if err != nil {
		return err
	}
	defer func() {
		_ = f.Close()
	}()

	if err := syscall.Flock(int(f.Fd()), syscall.LOCK_EX); err != nil {
		return err
	}
	defer func() {
		_ = syscall.Flock(int(f.Fd()), syscall.LOCK_UN)
	}()

	if err := f.Truncate(0); err != nil {
		return err
	}

	if _, err := f.Seek(0, 0); err != nil {
		return err
	}

	enc := json.NewEncoder(f)
	enc.SetEscapeHTML(false)
	enc.SetIndent("", "  ")
	return enc.Encode(cfg)
}

func (c *stuConfig) addTunnel(t stuTunnelConfig) error {
	latest, err := loadConfig()
	if err != nil {
		return err
	}

	for _, existing := range latest.Tunnels {
		if existing.Name == t.Name {
			return fmt.Errorf("tunnel with name %s already exists", t.Name)
		}
	}

	latest.Tunnels = append(latest.Tunnels, t)
	return saveConfig(latest)
}

func (c *stuConfig) updateTunnel(t stuTunnelConfig) error {
	return c.renameTunnel(t.Name, t)
}

func (c *stuConfig) renameTunnel(oldName string, t stuTunnelConfig) error {
	latest, err := loadConfig()
	if err != nil {
		return err
	}

	if oldName != t.Name {
		for _, existing := range latest.Tunnels {
			if existing.Name == t.Name {
				return fmt.Errorf("tunnel with name %s already exists", t.Name)
			}
		}
	}

	found := false
	for i, existing := range latest.Tunnels {
		if existing.Name == oldName {
			latest.Tunnels[i] = t
			found = true
			break
		}
	}

	if !found {
		return fmt.Errorf("tunnel with name %s not found", oldName)
	}

	return saveConfig(latest)
}

func (c *stuConfig) deleteTunnel(name string) error {
	latest, err := loadConfig()
	if err != nil {
		return err
	}

	found := false
	for i, t := range latest.Tunnels {
		if t.Name == name {
			latest.Tunnels = append(latest.Tunnels[:i], latest.Tunnels[i+1:]...)
			found = true
			break
		}
	}

	if !found {
		return fmt.Errorf("tunnel with name %s not found", name)
	}

	return saveConfig(latest)
}

func (c *stuConfig) getTunnel(name string) (stuTunnelConfig, bool) {
	for _, t := range c.Tunnels {
		if t.Name == name {
			return t, true
		}
	}

	return stuTunnelConfig{}, false
}

func (c *stuConfig) atomicUpdate(name string, updater func(*stuTunnelConfig)) error {
	path := getConfigPath()
	f, err := os.OpenFile(path, os.O_RDWR|os.O_CREATE, 0600)
	if err != nil {
		return err
	}
	defer func() {
		_ = f.Close()
	}()

	if err := syscall.Flock(int(f.Fd()), syscall.LOCK_EX); err != nil {
		return err
	}
	defer func() {
		_ = syscall.Flock(int(f.Fd()), syscall.LOCK_UN)
	}()

	var cfg stuConfig
	if err := json.NewDecoder(f).Decode(&cfg); err != nil {
		return err
	}

	found := false
	for i, t := range cfg.Tunnels {
		if t.Name == name {
			updater(&cfg.Tunnels[i])
			found = true
			break
		}
	}

	if !found {
		return fmt.Errorf("not found")
	}

	if err := f.Truncate(0); err != nil {
		return err
	}

	if _, err := f.Seek(0, 0); err != nil {
		return err
	}

	enc := json.NewEncoder(f)
	enc.SetEscapeHTML(false)
	enc.SetIndent("", "  ")
	return enc.Encode(&cfg)
}

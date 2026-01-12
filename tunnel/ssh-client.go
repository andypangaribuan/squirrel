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
	"io"
	"net"
	"os"
	"os/exec"
	"os/user"
	"path/filepath"
	"strings"
	"sync"
	"time"

	"golang.org/x/crypto/ssh"
)

type sshTunnel struct {
	cfg      *stuTunnelConfig
	client   *ssh.Client
	listener net.Listener
	stopOnce sync.Once
	stopChan chan struct{}
	doneChan chan error
}

func expandPath(path string) string {
	if strings.HasPrefix(path, "~/") {
		home, _ := os.UserHomeDir()
		return filepath.Join(home, path[2:])
	}

	return path
}

func startSSHTunnelGo(cfg *stuTunnelConfig) (*sshTunnel, error) {
	user, addr := parseHost(cfg.Host)

	authMethods := []ssh.AuthMethod{}
	if cfg.Password != "" {
		authMethods = append(authMethods, ssh.Password(cfg.Password))
	}

	if cfg.IdentityFile != "" {
		keyPath := expandPath(cfg.IdentityFile)
		key, err := os.ReadFile(keyPath)
		if err != nil {
			return nil, fmt.Errorf("failed to read identity file: %w", err)
		}

		signer, err := ssh.ParsePrivateKey(key)
		if err != nil {
			return nil, fmt.Errorf("failed to parse private key: %w", err)
		}

		authMethods = append(authMethods, ssh.PublicKeys(signer))
	}

	clientConfig := &ssh.ClientConfig{
		User:            user,
		Auth:            authMethods,
		HostKeyCallback: ssh.InsecureIgnoreHostKey(),
		Timeout:         5 * time.Second,
	}

	var conn net.Conn
	var err error

	if cfg.ProxyCommand != "" {
		conn, err = dialWithProxy(cfg.ProxyCommand, user, addr)
	} else {
		conn, err = net.DialTimeout("tcp", addr, clientConfig.Timeout)
	}

	if err != nil {
		return nil, fmt.Errorf("failed to connect to host: %w", err)
	}

	sshConn, chans, reqs, err := ssh.NewClientConn(conn, addr, clientConfig)
	if err != nil {
		_ = conn.Close()
		return nil, fmt.Errorf("failed to create ssh connection: %w", err)
	}

	client := ssh.NewClient(sshConn, chans, reqs)

	listener, err := net.Listen("tcp", "127.0.0.1:"+cfg.LocalPort)
	if err != nil {
		_ = client.Close()
		return nil, fmt.Errorf("failed to listen on local port: %w", err)
	}

	t := &sshTunnel{
		cfg:      cfg,
		client:   client,
		listener: listener,
		stopChan: make(chan struct{}),
		doneChan: make(chan error, 1),
	}

	go t.run()
	return t, nil
}

func (t *sshTunnel) run() {
	var wg sync.WaitGroup
	errChan := make(chan error, 1)

	go func() {
		for {
			localConn, err := t.listener.Accept()
			if err != nil {
				select {
				case <-t.stopChan:
				default:
					errChan <- err
				}
				return
			}

			wg.Add(1)
			go func() {
				defer wg.Done()
				t.handleConnection(localConn)
			}()
		}
	}()

	ticker := time.NewTicker(5 * time.Second)
	defer ticker.Stop()

	for {
		select {
		case <-t.stopChan:
			t.cleanup()
			wg.Wait()
			t.doneChan <- nil
			return

		case err := <-errChan:
			t.cleanup()
			wg.Wait()
			t.doneChan <- err
			return

		case <-ticker.C:
			_, _, err := t.client.SendRequest("keepalive@openssh.com", true, nil)
			if err != nil {
				t.cleanup()
				wg.Wait()
				t.doneChan <- fmt.Errorf("keepalive failed: %w", err)
				return
			}
		}
	}
}

func (t *sshTunnel) handleConnection(localConn net.Conn) {
	defer func() {
		_ = localConn.Close()
	}()

	remoteConn, err := t.client.Dial("tcp", t.cfg.RemoteAddr)
	if err != nil {
		return
	}
	defer func() {
		_ = remoteConn.Close()
	}()

	done := make(chan struct{}, 2)
	go func() {
		_, _ = io.Copy(localConn, remoteConn)
		done <- struct{}{}
	}()
	go func() {
		_, _ = io.Copy(remoteConn, localConn)
		done <- struct{}{}
	}()

	select {
	case <-done:
	case <-t.stopChan:
	}
}

func (t *sshTunnel) Stop() {
	t.stopOnce.Do(func() {
		close(t.stopChan)
		_ = t.listener.Close()
		_ = t.client.Close()
	})
}

func (t *sshTunnel) Wait() <-chan error {
	return t.doneChan
}

func (t *sshTunnel) cleanup() {
	_ = t.listener.Close()
	_ = t.client.Close()
}

func parseHost(host string) (username, addr string) {
	username = os.Getenv("USER")
	if username == "" {
		if u, err := user.Current(); err == nil {
			username = u.Username
		}
	}

	addr = host
	if i := strings.Index(host, "@"); i >= 0 {
		username = host[:i]
		addr = host[i+1:]
	}

	if !strings.Contains(addr, ":") {
		addr = addr + ":22"
	}

	return
}

func dialWithProxy(proxyCmd, user, addr string) (net.Conn, error) {
	host, port, _ := net.SplitHostPort(addr)
	cmdStr := strings.ReplaceAll(proxyCmd, "%h", host)
	cmdStr = strings.ReplaceAll(cmdStr, "%p", port)
	cmdStr = strings.ReplaceAll(cmdStr, "%r", user)

	cmd := exec.Command("/bin/sh", "-c", cmdStr)

	stdin, err := cmd.StdinPipe()
	if err != nil {
		return nil, err
	}

	stdout, err := cmd.StdoutPipe()
	if err != nil {
		return nil, err
	}

	if err := cmd.Start(); err != nil {
		return nil, err
	}

	return &proxyConn{
		Writer: stdin,
		Reader: stdout,
		cmd:    cmd,
	}, nil
}

type proxyConn struct {
	io.Writer
	io.Reader
	cmd *exec.Cmd
}

func (p *proxyConn) Close() error {
	_ = p.cmd.Process.Kill()
	return p.cmd.Wait()
}

func (p *proxyConn) LocalAddr() net.Addr                { return &net.TCPAddr{} }
func (p *proxyConn) RemoteAddr() net.Addr               { return &net.TCPAddr{} }
func (p *proxyConn) SetDeadline(t time.Time) error      { return nil }
func (p *proxyConn) SetReadDeadline(t time.Time) error  { return nil }
func (p *proxyConn) SetWriteDeadline(t time.Time) error { return nil }

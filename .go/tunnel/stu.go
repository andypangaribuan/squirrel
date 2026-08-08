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
	"net"
	"os/exec"
	"sync"

	"github.com/charmbracelet/bubbles/list"
	"golang.org/x/crypto/ssh"
)

type stuTunnelConfig struct {
	Name         string `json:"name"`
	Host         string `json:"host"`
	Password     string `json:"password,omitempty"`
	IdentityFile string `json:"identity_file,omitempty"`
	ProxyCommand string `json:"proxy_command,omitempty"`
	RemoteAddr   string `json:"remote_addr"`
	LocalPort    string `json:"local_port"`
	Actions      string `json:"actions,omitempty"` // "ssh", "tunnel", or "ssh,tunnel"
	PID          int    `json:"pid,omitempty"`
	SshPid       int    `json:"ssh_pid,omitempty"`
	Status       string `json:"status,omitempty"`
}

type stuConfig struct {
	Tunnels []stuTunnelConfig `json:"tunnels"`
}

type sshTunnel struct {
	cfg      *stuTunnelConfig
	client   *ssh.Client
	listener net.Listener
	stopOnce sync.Once
	stopChan chan struct{}
	doneChan chan error
}

type stuFormFinishedMsg struct{}

type stuTunnelItem struct {
	config     stuTunnelConfig
	running    bool
	sshMode    bool
	maxNameLen int
	maxPortLen int
}

type stuItemModel struct {
	sshMode      bool
	list         list.Model
	state        sessionState
	selected     stuTunnelItem
	actionChoice int
	actions      []string
	quitting     bool
	lastWidth    int
	lastHeight   int
	pendingCmd   *exec.Cmd
}

type stuProxyConn struct {
	io.Writer
	io.Reader
	cmd *exec.Cmd
}

type stuStdioConn struct {
	w io.Writer
	r io.Reader
}

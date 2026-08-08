package tunnel

import (
	"fmt"
	"log"
	"net"
	"os"
	"os/exec"
	"os/signal"
	"strings"
	"syscall"

	"golang.org/x/crypto/ssh"
	"golang.org/x/term"
)

func runAccess(name string) {
	cfg, err := loadConfig()
	if err != nil {
		log.Fatalf("Failed to load config: %v", err)
	}

	t, ok := cfg.getTunnel(name)
	if !ok {
		log.Fatalf("Tunnel '%s' not found", name)
	}

	authMethods := []ssh.AuthMethod{}
	if t.Password != "" {
		authMethods = append(authMethods, ssh.Password(t.Password))
	}

	user, host := parseHost(t.Host)
	if user == "" {
		if u := os.Getenv("USER"); u != "" {
			user = u
		}
	}

	port := "22"
	if strings.Contains(host, ":") {
		h, p, err := net.SplitHostPort(host)
		if err == nil {
			host = h
			port = p
		}
	}

	sshConfig := &ssh.ClientConfig{
		User:            user,
		Auth:            authMethods,
		HostKeyCallback: ssh.InsecureIgnoreHostKey(), // Match -o StrictHostKeyChecking=no
		Timeout:         0,                           // No timeout for interactive
	}

	var client *ssh.Client

	if t.ProxyCommand != "" {
		cmdStr := t.ProxyCommand
		cmdStr = strings.ReplaceAll(cmdStr, "%h", host)
		cmdStr = strings.ReplaceAll(cmdStr, "%p", port)
		cmdStr = strings.ReplaceAll(cmdStr, "%r", user)

		proxyCmd := exec.Command("sh", "-c", cmdStr)
		stdin, err := proxyCmd.StdinPipe()
		if err != nil {
			log.Fatalf("Failed to get proxy stdin: %v", err)
		}

		stdout, err := proxyCmd.StdoutPipe()
		if err != nil {
			log.Fatalf("Failed to get proxy stdout: %v", err)
		}

		if err := proxyCmd.Start(); err != nil {
			log.Fatalf("Failed to start proxy command: %v", err)
		}

		c, chans, reqs, err := ssh.NewClientConn(&stuStdioConn{stdin, stdout}, fmt.Sprintf("%s:%s", host, port), sshConfig)
		if err != nil {
			log.Fatalf("Failed to connect via proxy: %v", err)
		}

		client = ssh.NewClient(c, chans, reqs)
	} else {
		var err error
		client, err = ssh.Dial("tcp", fmt.Sprintf("%s:%s", host, port), sshConfig)
		if err != nil {
			log.Fatalf("Failed to dial: %v", err)
		}
	}
	defer func() {
		_ = client.Close()
	}()

	session, err := client.NewSession()
	if err != nil {
		log.Fatalf("Failed to create session: %v", err)
	}
	defer func() {
		_ = session.Close()
	}()

	fd := int(os.Stdin.Fd())
	if term.IsTerminal(fd) {
		oldState, err := term.MakeRaw(fd)
		if err != nil {
			log.Printf("Warning: failed to make terminal raw: %v", err)
		} else {
			defer func() { _ = term.Restore(fd, oldState) }()
		}

		w, h, err := term.GetSize(fd)
		if err == nil {
			if err := session.RequestPty("xterm-256color", h, w, ssh.TerminalModes{
				ssh.ECHO:          1,
				ssh.TTY_OP_ISPEED: 14400,
				ssh.TTY_OP_OSPEED: 14400,
			}); err != nil {
				log.Printf("Warning: request pty failed: %v", err)
			}
		}

		go func() {
			sig := make(chan os.Signal, 1)
			signal.Notify(sig, syscall.SIGWINCH)
			for range sig {
				w, h, err := term.GetSize(fd)
				if err == nil {
					_ = session.WindowChange(h, w)
				}
			}
		}()
	}

	session.Stdin = os.Stdin
	session.Stdout = os.Stdout
	session.Stderr = os.Stderr

	if err := session.Shell(); err != nil {
		log.Fatalf("Failed to start shell: %v", err)
	}

	if err := session.Wait(); err != nil {
		if e, ok := err.(*ssh.ExitError); ok {
			os.Exit(e.ExitStatus())
		}
	}
}

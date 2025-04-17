/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package util

import (
	"bytes"
	"fmt"
	"os"
	"os/exec"
	"strings"
)

func InteractiveTerminal(workingDirectory string, script string, envs ...string) error {
	return execInteractiveTerminal(false, workingDirectory, script, envs...)
}

func ZInteractiveTerminal(workingDirectory string, script string, envs ...string) error {
	return execInteractiveTerminal(true, workingDirectory, script, envs...)
}

func Terminal(workingDirectory string, script string) (out string, err *string) {
	return execTerminal(false, workingDirectory, script)
}

func ZTerminal(workingDirectory string, script string) (out string, err *string) {
	return execTerminal(true, workingDirectory, script)
}

func execTerminal(withZshrc bool, workingDirectory string, script string) (out string, err *string) {
	if workingDirectory != "" {
		script = fmt.Sprintf("cd %v; %v", workingDirectory, script)
	}

	if withZshrc {
		script = "set -a; source ~/.zshrc; set +a; " + script
	}

	var stdout, stderr bytes.Buffer
	cmd := exec.Command("bash", "-c", script)
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	_ = cmd.Run()

	out = stdout.String()
	outErr := strings.TrimSpace(stderr.String())
	if outErr != "" {
		err = &outErr
	}

	return
}

func execInteractiveTerminal(withZshrc bool, workingDirectory string, script string, envs ...string) error {
	if workingDirectory != "" {
		script = fmt.Sprintf("cd %v; %v", workingDirectory, script)
	}

	if withZshrc {
		script = "set -a; source ~/.zshrc; set +a; " + script
	}

	cmd := exec.Command("sh", "-c", script)
	env := os.Environ()
	if len(envs) > 0 {
		env = append(env, envs...)
	}
	cmd.Env = env

	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin

	err := cmd.Run()
	if err != nil {
		return err
	}

	if err := cmd.Wait(); err != nil {
		return err
	}

	return nil
}

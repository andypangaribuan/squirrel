/*
 * Copyright (c) 2026.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package taskfile

import (
	"bufio"
	"fmt"
	"os"
	"os/exec"
	"regexp"
	"squirrel/util"
	"strings"
)

func cliTaskfileExecute() {
	if os.Getenv("TASKFILE_EXECUTOR") != "" {
		return
	}

	taskfilePath := ".taskfile"
	if _, err := os.Stat(taskfilePath); os.IsNotExist(err) {
		fmt.Fprintf(os.Stderr, "Error: %s not found\n", taskfilePath)
		return
	}

	args := os.Args[3:]
	if len(args) == 0 {
		executeHelp(taskfilePath)
		return
	}

	executeTask(taskfilePath, args)
}

func executeHelp(path string) {
	file, err := os.Open(path)
	if err != nil {
		return
	}

	defer func() {
		_ = file.Close()
	}()

	var tasks []stuTaskItem
	scanner := bufio.NewScanner(file)
	reFunc := regexp.MustCompile(`^(?:function\s+)?([a-zA-Z0-9_-]+)(?:\s*\(\))?\s*\{`)

	var lastComment string
	for scanner.Scan() {
		line := strings.TrimSpace(scanner.Text())

		if after, ok := strings.CutPrefix(line, "#: "); ok {
			comment := after
			if comment == "space" {
				tasks = append(tasks, stuTaskItem{isSpace: true})
				lastComment = ""
				continue
			}
			lastComment = comment
			continue
		}

		if lastComment != "" {
			matches := reFunc.FindStringSubmatch(line)
			if len(matches) > 1 {
				name := matches[1]
				if name != "help" {
					tasks = append(tasks, stuTaskItem{
						name:        name,
						description: lastComment,
					})
				}
				lastComment = ""
			}
		}
	}

	globalMaxLen := 0
	for _, task := range tasks {
		if !task.isSpace && len(task.name) > globalMaxLen {
			globalMaxLen = len(task.name)
		}
	}

	var parsedTasks []stuTaskParsed

	for _, task := range tasks {
		p1 := task.description
		p2 := ""

		if !task.isSpace {
			if before, after, ok := strings.Cut(task.description, "#:"); ok {
				p1 = strings.TrimSpace(before)
				p2 = strings.TrimSpace(after)
			}
		}

		parsedTasks = append(parsedTasks, stuTaskParsed{task, p1, p2})
	}

	var (
		blocks       [][]stuTaskParsed
		currentBlock []stuTaskParsed
		p2Separator  = util.ColorYellow("»")
	)

	for _, task := range parsedTasks {
		if task.isSpace {
			if len(currentBlock) > 0 {
				blocks = append(blocks, currentBlock)
			}

			blocks = append(blocks, []stuTaskParsed{task})
			currentBlock = nil
		} else {
			currentBlock = append(currentBlock, task)
		}
	}

	if len(currentBlock) > 0 {
		blocks = append(blocks, currentBlock)
	}

	if len(blocks) > 0 {
		fmt.Println(util.ColorBoldGreen("commands:"))
	}

	for _, block := range blocks {
		if len(block) == 1 && block[0].isSpace {
			fmt.Println("")
			continue
		}

		localMaxP1Len := 0
		for _, task := range block {
			if len(task.p1) > localMaxP1Len {
				localMaxP1Len = len(task.p1)
			}
		}

		for _, task := range block {
			if task.p2 != "" {
				fmt.Printf("  %-*s   %-*s   %s %s\n", globalMaxLen, task.name, localMaxP1Len, task.p1, p2Separator, task.p2)
			} else {
				fmt.Printf("  %-*s   %s\n", globalMaxLen, task.name, task.p1)
			}
		}
	}
}

func executeTask(path string, args []string) {
	script := fmt.Sprintf("source %s; if type \"$1\" >/dev/null 2>&1; then \"$@\"; else echo \"command not found: $1\"; exit 127; fi", path)

	cmd := exec.Command("bash", "-c", script, "taskfile")
	cmd.Args = append(cmd.Args, args...)

	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	cmd.Stdin = os.Stdin

	cmd.Env = append(os.Environ(), "TASKFILE_EXECUTOR=1")

	err := cmd.Run()
	if err != nil {
		if exitErr, ok := err.(*exec.ExitError); ok {
			os.Exit(exitErr.ExitCode())
		}

		os.Exit(1)
	}
}

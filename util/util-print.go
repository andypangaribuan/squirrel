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
	"fmt"
	"os"
	"strings"
)

func Print(help string) {
	fmt.Printf("%v\n\n", strings.TrimSpace(help))
}

func PrintThenExit(help string, isError ...bool) {
	fmt.Printf("%v\n\n", strings.TrimSpace(help))
	if len(isError) > 0 && isError[0] {
		os.Exit(1)
	}
	os.Exit(0)
}

func UnknownCommand(remainsCommand string, helpMessage string) {
	remainsCommand = strings.TrimSpace(remainsCommand)
	msg := helpMessage
	if remainsCommand != "" {
		msg = fmt.Sprintf("unknown: %v\n\n%v", remainsCommand, msg)
	}

	PrintThenExit(msg, true)
}

/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package clitaskfile

import (
	"fmt"
	"os"
)

func getWorkingDirectory() string {
	workingDir, err := os.Getwd()
	if err != nil {
		fmt.Printf("%+v\n", err)
		os.Exit(1)
	}

	return workingDir
}

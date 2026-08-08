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

import "github.com/fatih/color"

const cReset = "\033[0m"
const cGreen = "\033[32m"

func ColorGreen(text string) string {
	return cGreen + text + cReset
}

var ColorBoldGreen = color.New(color.Bold, color.FgGreen).SprintFunc()
var ColorBoldRed = color.New(color.Bold, color.FgHiRed).SprintFunc()
var ColorRed = color.New(color.FgHiRed).SprintFunc()
var ColorYellow = color.New(color.FgHiYellow).SprintFunc()
var ColorCyan = color.New(color.FgHiCyan).SprintFunc()

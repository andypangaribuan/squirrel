/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package taskfile

type stuTaskfile struct {
	items            [][]any
	newLineAtIndexes []int
}

type stuTaskItem struct {
	name        string
	description string
	isSpace     bool
}

type stuTaskParsed struct {
	stuTaskItem
	p1 string
	p2 string
}

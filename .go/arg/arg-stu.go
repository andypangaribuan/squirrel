/*
 * Copyright (c) 2025.
 * Created by Andy Pangaribuan (iam.pangaribuan@gmail.com)
 * https://github.com/apangaribuan
 *
 * This product is protected by copyright and distributed under
 * licenses restricting copying, distribution and decompilation.
 * All Rights Reserved.
 */

package arg

type stuWatch struct {
	currentPath string
	helpMessage string
	rootMessage string
	items       []*stuWatchItem
}

type stuWatchItem struct {
	name     string
	alias    string
	callback func()
}

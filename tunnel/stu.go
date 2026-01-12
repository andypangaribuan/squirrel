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

import "github.com/charmbracelet/bubbles/list"

type formFinishedMsg struct{}

type tunnelItem struct {
	config     stuTunnelConfig
	running    bool
	maxNameLen int
	maxPortLen int
}

type itemModel struct {
	list         list.Model
	state        sessionState
	selected     tunnelItem
	actionChoice int
	actions      []string
	quitting     bool
	lastWidth    int
	lastHeight   int
}

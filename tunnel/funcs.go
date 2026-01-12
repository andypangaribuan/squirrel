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
	"time"

	tea "github.com/charmbracelet/bubbletea"
)

func tick() tea.Cmd {
	return tea.Tick(time.Second, func(t time.Time) tea.Msg {
		return tickMsg(t)
	})
}

func calculateMaxLengths(tunnels []stuTunnelConfig) (int, int) {
	maxN, maxP := 0, 0

	for _, t := range tunnels {
		if len(t.Name) > maxN {
			maxN = len(t.Name)
		}
		if len(t.LocalPort) > maxP {
			maxP = len(t.LocalPort)
		}
	}

	if maxP < 4 {
		maxP = 4
	}

	return maxN, maxP
}

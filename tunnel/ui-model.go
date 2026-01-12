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
	"fmt"
	"os/exec"

	"github.com/charmbracelet/bubbles/list"
	tea "github.com/charmbracelet/bubbletea"
	"github.com/charmbracelet/lipgloss"
)

func (m itemModel) Init() tea.Cmd {
	return tick()
}

func (m *itemModel) ResizeList() {
	h, v := docStyle.GetFrameSize()
	if m.list.Help.ShowAll {
		m.list.SetSize(m.lastWidth-h, m.lastHeight-v)
	} else {
		m.list.SetSize(m.lastWidth-h, m.lastHeight-v-2)
	}
}

func (m itemModel) Update(msg tea.Msg) (tea.Model, tea.Cmd) {
	switch msg := msg.(type) {
	case tea.KeyMsg:
		key := msg.String()

		switch m.state {
		case stateList:
			if key == "enter" {
				if m.list.FilterState() == list.Filtering {
					m.list.Update(msg)
				}
				if item, ok := m.list.SelectedItem().(tunnelItem); ok {
					m.selected = item
					m.state = stateActions
					m.actionChoice = 0
					m.updateActions()
					return m, nil
				}
			}

			if key == "a" && m.list.FilterState() != list.Filtering {
				m.state = stateForm
				return m, tea.ExecProcess(exec.Command("clear"), func(err error) tea.Msg {
					addTunnelLogic()
					return formFinishedMsg{}
				})
			}

			if key == "?" {
				m.list.SetShowHelp(!m.list.ShowHelp())
				m.list.Help.ShowAll = m.list.ShowHelp()
				m.ResizeList()
				return m, nil
			}

			if key == "esc" && m.list.Help.ShowAll {
				m.list.SetShowHelp(false)
				m.list.Help.ShowAll = false
				m.ResizeList()
				return m, nil
			}

			if key == "." && m.list.FilterState() != list.Filtering {
				stopAllTunnelsLogic()
				return m, nil
			}

			if key == "ctrl+c" || (key == "q" && m.list.FilterState() != list.Filtering) {
				m.quitting = true
				return m, tea.Quit
			}

		case stateActions:
			switch key {
			case "ctrl+c", "esc", "q":
				m.state = stateList
				return m, nil
			case "up", "k":
				if m.actionChoice > 0 {
					m.actionChoice--
				}
			case "down", "j":
				if m.actionChoice < len(m.actions)-1 {
					m.actionChoice++
				}
			case "enter":
				action := m.actions[m.actionChoice]
				return m.handleAction(action)
			}
		}

	case tea.WindowSizeMsg:
		m.lastWidth = msg.Width
		m.lastHeight = msg.Height
		m.ResizeList()
		return m, nil

	case formFinishedMsg:
		m.state = stateList
		return m, nil

	case tickMsg:
		cfg, err := loadConfig()
		if err == nil {
			maxN, maxP := calculateMaxLengths(cfg.Tunnels)

			if m.state == stateActions {
				if t, found := cfg.getTunnel(m.selected.config.Name); found {
					isRunning := isTunnelRunning(t.PID)
					displayStatus := syncStatus(t.Name, isRunning, t.Status, t.LocalPort)

					t.Status = displayStatus
					m.selected = tunnelItem{config: t, running: isRunning, maxNameLen: maxN, maxPortLen: maxP}
					m.updateActions()
				}
			}

			if m.state == stateList {
				var selectedName string
				if item, ok := m.list.SelectedItem().(tunnelItem); ok {
					selectedName = item.config.Name
				}

				var currentItems []list.Item
				for _, t := range cfg.Tunnels {
					isRunning := isTunnelRunning(t.PID)
					displayStatus := syncStatus(t.Name, isRunning, t.Status, t.LocalPort)

					t.Status = displayStatus
					item := tunnelItem{config: t, running: isRunning, maxNameLen: maxN, maxPortLen: maxP}
					currentItems = append(currentItems, item)
				}
				m.list.SetItems(currentItems)

				if selectedName != "" {
					for i, item := range m.list.Items() {
						if ti, ok := item.(tunnelItem); ok && ti.config.Name == selectedName {
							m.list.Select(i)
							break
						}
					}
				}
			}
		}
		return m, tick()
	}

	if m.state == stateList {
		var cmd tea.Cmd
		m.list, cmd = m.list.Update(msg)
		return m, cmd
	}

	return m, nil
}

func (m itemModel) View() string {
	if m.quitting || m.state == stateForm {
		return ""
	}

	if m.state == stateActions {
		statusLine := stoppedStyle.Render("stopped")
		status := m.selected.config.Status
		if status == "disconnected-ready" {
			status = "disconnected"
		}

		if status != "" {
			switch status {
			case "connected":
				statusLine = runningStyle.Render("connected")
			case "disconnected":
				statusLine = errorStyle.Render("disconnected")
			case "reconnecting":
				statusLine = pendingStyle.Render("reconnecting")
			default:
				statusLine = runningStyle.Render(status)
			}
		}

		s := fmt.Sprintf("\n  --- Tunnel: %s %s ---\n\n", m.selected.config.Name, statusLine)
		for i, action := range m.actions {
			cursor := "  "
			style := lipgloss.NewStyle()
			if m.actionChoice == i {
				cursor = "> "
				style = selectedStyle
			}
			s += fmt.Sprintf("  %s%s\n", cursor, style.Render(action))
		}
		s += "\n  (esc to go back)\n"
		return docStyle.Render(s)
	}

	if m.list.Help.ShowAll {
		return docStyle.Render(m.list.View())
	}

	return docStyle.Render(m.list.View() + "\n\n  " + lipgloss.NewStyle().Foreground(lipgloss.Color("241")).Render("↑↓ • . stop all • / filter • q quit • ? more"))
}

func (m *itemModel) updateActions() {
	if m.selected.running {
		m.actions = []string{"Stop", "Update", "Delete"}
	} else {
		m.actions = []string{"Start", "Update", "Delete"}
	}
	if m.actionChoice >= len(m.actions) {
		m.actionChoice = 0
	}
}

func (m itemModel) handleAction(action string) (tea.Model, tea.Cmd) {
	name := m.selected.config.Name
	switch action {
	case "Start":
		startTunnelLogic(name)
		m.state = stateList
		return m, nil
	case "Stop":
		stopTunnelLogic(name)
		m.state = stateList
		return m, nil
	case "Delete":
		m.state = stateForm
		return m, tea.ExecProcess(exec.Command("clear"), func(err error) tea.Msg {
			deleteTunnelLogic(name)
			return formFinishedMsg{}
		})
	case "Update":
		m.state = stateForm
		return m, tea.ExecProcess(exec.Command("clear"), func(err error) tea.Msg {
			updateTunnelLogic(name)
			return formFinishedMsg{}
		})
	}
	return m, nil
}

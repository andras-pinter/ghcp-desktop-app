/** Centralized English strings for Chuck. */

export const strings = {
  app: {
    name: "Chuck",
    tagline: "Break the barrier.",
  },

  auth: {
    title: "Chuck",
    subtitle: "Sign in with GitHub to get started.",
    signIn: "Sign in with GitHub →",
    orEnterCode: "or enter code",
    yourCode: "Your code:",
    copyCode: "Copy Code 📋",
    waitingForAuth: "Waiting for auth...",
    requiresSubscription: "Requires an active Copilot subscription",
  },

  sidebar: {
    newChat: "+ New Chat",
    searchPlaceholder: "Search...",
    favourites: "★ Favourites",
    projects: "📁 Projects",
    agents: "🤖 Agents",
    today: "📅 Today",
    yesterday: "📅 Yesterday",
    lastWeek: "📅 Last 7 Days",
    older: "📅 Older",
    skills: "⚡ Skills",
    sources: "🔀 Sources",
    settings: "⚙️ Settings",
    noFavourites: "No favourites yet",
    noProjects: "No projects yet",
    noConversations: "No conversations yet",
  },

  chat: {
    newConversation: "New Conversation",
    startConversation: "Start a conversation",
    askAnything: "Ask anything — coding, research, brainstorming.",
    messagePlaceholder: "Message Copilot...",
    send: "Send ➤",
    attachFile: "Attach file",
    webSearch: "Web search",
    you: "You",
    copilot: "Copilot",
  },

  status: {
    online: "● Online",
    offline: "● Offline",
  },
} as const;

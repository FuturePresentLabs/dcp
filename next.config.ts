import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Fab Sites publishes immutable files; preserve the default Worker build.
  ...(process.env.FAB_STATIC_EXPORT === "1" ? { output: "export" as const } : {}),
};

export default nextConfig;

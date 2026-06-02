import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  output: "export",
  reactStrictMode: false,
  transpilePackages: ["@supportflow/shared", "@supportflow/ui"],
  images: {
    unoptimized: true
  }
};

export default nextConfig;

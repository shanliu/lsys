import { defineConfig } from "@rspack/cli";
import { rspack } from "@rspack/core";
import { ReactRefreshRspackPlugin } from "@rspack/plugin-react-refresh";
import { tanstackRouter } from '@tanstack/router-plugin/rspack';
import Dotenv from 'dotenv-webpack';
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const __dirname = dirname(fileURLToPath(import.meta.url));
const isDev = process.env.NODE_ENV === "development";

// 读取 tsr.config.json 保持路由配置一致
const tsrConfig = require('./tsr.config.json');

// Target browsers, see: https://github.com/browserslist/browserslist
const targets = ["last 2 versions", "> 0.2%", "not dead", "Firefox ESR"];

// 应用配置
const apps = [
	{ name: 'home', entry: 'apps/home/index.tsx', output: 'index.html', title: '内部应用管理平台', splitChunks: false },
	{ name: 'main', entry: 'apps/main/index.tsx', output: 'app.html', title: '内部应用管理平台', splitChunks: true },
];

export default defineConfig({
	context: __dirname,
	entry: Object.fromEntries(
		apps.map(app => [app.name, resolve(__dirname, 'src', app.entry)])
	),
	mode: isDev ? 'development' : 'production',
	resolve: {
		extensions: ["...", ".ts", ".tsx", ".jsx"],
		alias: {
			'@': resolve(__dirname, './src'),
			'@shared': resolve(__dirname, './src/shared'),
			'@apps': resolve(__dirname, './src/apps'),
		}
	},
	devServer: {
		...(isDev && {
			// Multi-page fallback:
			// - / or /index.html -> index.html (home)
			// - other routes     -> app.html (main SPA)
			historyApiFallback: {
				rewrites: [
					{ from: /^(\/|\/index\.html(\?.*)?)$/, to: "/index.html" },
					{ from: /./, to: "/app.html" },
				],
			},
		}),
	},
	module: {
		rules: [
			{
				test: /\.css$/,
				use: ["postcss-loader"],
				type: "css",
			},
			{
				test: /\.(png?|svg?)$/,
				type: "asset"
			},
			{
				test: /\.(jsx?|tsx?)$/,
				use: [
					{
						loader: "builtin:swc-loader",
						options: {
							jsc: {
								parser: {
									syntax: "typescript",
									tsx: true
								},
								transform: {
									react: {
										runtime: "automatic",
										development: isDev,
										refresh: isDev
									}
								}
							},
							env: { targets }
						}
					}
				]
			}
		]
	},
	plugins: [
		new Dotenv({
			systemvars: true,
			path: `./.env.${process.env.NODE_ENV || 'development'}`
		}),
		tanstackRouter({
			target: 'react',
			autoCodeSplitting: tsrConfig.autoCodeSplitting,
			routesDirectory: resolve(__dirname, tsrConfig.routesDirectory),
			generatedRouteTree: resolve(__dirname, tsrConfig.generatedRouteTree),
		}),
		...apps.map(app => new rspack.HtmlRspackPlugin({
			template: resolve(__dirname, 'public/template.html'),
			filename: app.output,
			chunks: [app.name],
			title: app.title,
			favicon: resolve(__dirname, 'public/favicon.ico'),
			inject: 'body',
		})),
		isDev ? new ReactRefreshRspackPlugin() : null
	].filter(Boolean),
	optimization: {
		runtimeChunk: {
			name: (entrypoint) => `runtime-${entrypoint.name}`
		},
		splitChunks: {
			chunks: (chunk) => apps.find(app => app.name === chunk.name)?.splitChunks ?? false,
			minSize: 10000,
			maxSize: 150000,
			cacheGroups: {
				react: {
					test: /[\\/]node_modules[\\/](react|react-dom)[\\/]/,
					name: 'react',
					priority: 40
				},
				router: {
					test: /[\\/]node_modules[\\/](@tanstack[\\/]react-router|@tanstack[\\/]router)[\\/]/,
					name: 'router',
					priority: 35
				},
				query: {
					test: /[\\/]node_modules[\\/]@tanstack[\\/]react-query[\\/]/,
					name: 'query',
					priority: 35
				},
				radix: {
					test: /[\\/]node_modules[\\/]@radix-ui[\\/]/,
					name: 'radix-ui',
					priority: 30
				},
				vendors: {
					test: /[\\/]node_modules[\\/]/,
					name: 'vendors',
					priority: 20
				},
				ui: {
					test: /[\\/]src[\\/]shared[\\/]components[\\/]ui[\\/]/,
					name: 'ui-components',
					priority: 15
				},
				shared: {
					test: /[\\/]src[\\/]shared[\\/]/,
					name: 'shared',
					priority: 10
				}
			}
		},
		minimizer: [
			new rspack.SwcJsMinimizerRspackPlugin(),
			new rspack.LightningCssMinimizerRspackPlugin({
				minimizerOptions: { targets }
			})
		]
	},
	experiments: {
		css: true
	},
	output: {
		filename: '[name].bundle.js',
		publicPath: '/',
	}
});

let webpack = require('webpack');
let path = require("path");
let HtmlWebpackPlugin = require('html-webpack-plugin');
let { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const mode = process.env.npm_lifecycle_script.indexOf("mode=production") > 0 ? "production" : "development";
const useCdn = process.env.npm_lifecycle_script.indexOf("env=useCdn") > 0;

let jslib_oauth = jslib_wechat_login = jslib_app = ["config.js"];
let jslib_index = [];
let externals = {};
let devtool = "source-map"
let node_modules = path.resolve(__dirname, './node_modules');
if (mode === "production") {
    node_modules = null;
    devtool = false;
    if (useCdn) {
        const jslib_base = [
            "https://npm.elemecdn.com/react@18.2.0/umd/react.production.min.js",
            "https://npm.elemecdn.com/react-dom@18.2.0/umd/react-dom.production.min.js",
        ];
        jslib_index = [
            ...jslib_base,
            'https://npm.elemecdn.com/starback@2.1.1/dist/starback.global.js'
        ];
        jslib_app = [
            ...jslib_base,
            "https://npm.elemecdn.com/@mui/material@5.11.10/umd/material-ui.production.min.js",
            "https://cdnjs.cloudflare.com/ajax/libs/axios/1.1.3/axios.min.js",
            "https://cdnjs.cloudflare.com/ajax/libs/dayjs/1.11.6/dayjs.min.js",
            "config.js"
        ];
        jslib_oauth = [
            ...jslib_base,
            "https://npm.elemecdn.com/@mui/material@5.11.10/umd/material-ui.production.min.js",
            "https://cdnjs.cloudflare.com/ajax/libs/axios/1.1.3/axios.min.js",
            "config.js"
        ];
        jslib_wechat_login = [
            ...jslib_base,
            "config.js"
        ];
        externals = {
            'react': 'React',
            'react-dom': 'ReactDOM',
            '@mui/material': 'MaterialUI',
            'starback': 'Starback',
            'axios': 'axios',
            'dayjs': 'dayjs'
        };
    }
}

const version = new Date().getTime()
const output_path = __dirname + '/public';
let plugins = [
    new CopyWebpackPlugin({
        patterns: [
            {
                from: './src/config.js',
                to: output_path + "/config.js",
            },
        ],
    }),
    //多页面配置
    new HtmlWebpackPlugin({
        filename: 'index.html',
        chunks: ['index'],//需要导入的JS
        templateParameters: {
            js: jslib_index,
        },
        favicon: __dirname + './../server/examples/lsys-actix-web/static/favicon.ico',
        template: __dirname + "/src/page_index/index.html",
        nodeModules: node_modules
    }),
    new HtmlWebpackPlugin({
        filename: 'app.html',
        chunks: ['app'],//需要导入的JS
        templateParameters: {
            js: jslib_app,
        },
        favicon: __dirname + './../server/examples/lsys-actix-web/static/favicon.ico',
        template: __dirname + "/src/page_app/app.html",
        nodeModules: node_modules
    }),
    new HtmlWebpackPlugin({
        filename: 'oauth.html',
        chunks: ['oauth'],//需要导入的JS
        templateParameters: {
            js: jslib_oauth,
        },
        favicon: __dirname + './../server/examples/lsys-actix-web/static/favicon.ico',
        template: __dirname + "/src/page_oauth/oauth_app.html",
        nodeModules: node_modules
    }),
    new HtmlWebpackPlugin({
        filename: 'wechat-login.html',
        templateParameters: {
            js: jslib_wechat_login,
        },
        chunks: ['wechat_login'],//需要导入的JS
        template: __dirname + "/src/page_wechat_login/wechat_login.html",
        favicon: __dirname + './../server/examples/lsys-actix-web/static/favicon.ico',
        nodeModules: process.env.NODE_ENV !== 'production' ? path.resolve(__dirname, './node_modules') : false
    }),
];
if (process.env.npm_config_analyzer) {
    plugins.push(new BundleAnalyzerPlugin())
}

let config = {
    mode: mode,
    devtool: devtool,
    entry: {
        index: __dirname + `/src/page_index/index.js`,
        app: __dirname + `/src/page_app/app.js`,
        oauth: __dirname + `/src/page_oauth/oauth.js`,
        wechat_login: __dirname + `/src/page_wechat_login/wechat_login.js`
    },
    output: {
        path: output_path,
        filename: `js/[name].${version}.js`
    },
    externals: { 'siteConfig': "SiteConfig", ...externals },
    devServer: {
        static: path.join(__dirname, "public"),
        hot: true,
        liveReload: true,
        open: true,
        historyApiFallback: true
    },
    module: {
        rules: [
            {
                test: /(\.js)$/,
                use: {
                    loader: "babel-loader",
                    options: { presets: ['@babel/env', '@babel/preset-react'] }
                },
                exclude: /node_modules/
            },
            {
                test: /\.css$/,
                use: [
                    {
                        loader: "style-loader"
                    }, {
                        loader: "css-loader"
                    }
                ]
            },
            {
                test: /\.jpg|\.png|\.gif|\.webp$/,
                use: {
                    loader: "file-loader"
                }
            }
        ]
    },
    plugins: plugins,
    optimization: {
        runtimeChunk: true,
        splitChunks: {
            chunks: "async",
            minSize: 30000,
            cacheGroups: {
                defaultVendors: {
                    test: /[\\/]node_modules[\\/]/
                }
            }
        }
    },
};
module.exports = config;

let webpack = require('webpack');
let path = require("path");
let HtmlWebpackPlugin = require('html-webpack-plugin');
let { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');

let mode = process.env.npm_lifecycle_script.indexOf("mode=production") > 0 ? "production" : "development";
let version = new Date().getTime()
let node_modules = null;
let jslib = [
    "https://npm.elemecdn.com/react@18.2.0/umd/react.production.min.js",
    "https://npm.elemecdn.com/react-dom@18.2.0/umd/react-dom.production.min.js",
    "https://npm.elemecdn.com/@mui/material@5.11.10/umd/material-ui.production.min.js"
];
let externals = {
    'react': 'React',
    'react-dom': 'ReactDOM',
    '@mui/material': 'MaterialUI',
};
let devtool = "hidden-source-map";
if (mode !== "production") {
    jslib = [];
    externals = {};
}
node_modules = path.resolve(__dirname, '../node_modules');
devtool = "source-map"

let plugins = [
    //多页面配置
    new HtmlWebpackPlugin({
        filename: 'index.html',
        chunks: ['index'],//需要导入的JS
        templateParameters: {
            js: jslib,
        },
        favicon: __dirname + './../../server/examples/lsys-actix-web/static/favicon.ico',
        template: __dirname + "/src/app.html",
        nodeModules: node_modules
    }),
    new HtmlWebpackPlugin({
        filename: 'oauth.html',
        chunks: ['oauth'],//需要导入的JS
        templateParameters: {
            js: jslib,
        },
        favicon: __dirname + './../../server/examples/lsys-actix-web/static/favicon.ico',
        template: __dirname + "/src/oauth_app.html",
        nodeModules: node_modules
    })
];
if (process.env.npm_config_analyzer) {
    plugins.push(new BundleAnalyzerPlugin())
}

let config = {
    mode: mode,
    devtool: devtool,
    entry: {
        index: __dirname + `/src/app.js`,
        oauth: __dirname + `/src/oauth.js`,
    },
    output: {
        path: __dirname + '/../public',
        filename: `js/[name].${version}.js`
    },
    externals: externals,
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
            }
        ]
    },
    plugins: plugins,
    // optimization: {
    //     splitChunks: {
    //         chunks: 'all'
    //     }
    // },
};
module.exports = config;

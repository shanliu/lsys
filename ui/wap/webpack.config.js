let path = require("path");
let HtmlWebpackPlugin = require('html-webpack-plugin');
let { BundleAnalyzerPlugin } = require('webpack-bundle-analyzer');

let plugins = [
    //多页面配置
    new HtmlWebpackPlugin({
        filename: 'wechat-login.html',
        chunks: ['wechat'],//需要导入的JS
        template: __dirname + "/src/wechat/login.html",
        favicon: __dirname + './../../server/examples/lsys-actix-web/static/favicon.ico',
        nodeModules: process.env.NODE_ENV !== 'production' ? path.resolve(__dirname, '../node_modules') : false
    }),
];
if (process.env.npm_config_analyzer) {
    plugins.push(new BundleAnalyzerPlugin())
}
const version = new Date().getTime()
module.exports = {
    mode: 'development',
    entry: {
        wechat: __dirname + '/src/wechat/login.js',
    },
    output: {
        path: __dirname + '/../public/mobile/',
        filename: `js/[name].${version}.js`
    },
    externals: {
    },
    devtool: 'source-map',
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
    optimization: {
        splitChunks: {
            chunks: 'all'
        }
    },
};

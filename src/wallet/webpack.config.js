const HtmlWebpackPlugin = require('html-webpack-plugin');
const NodePolyfillPlugin = require('node-polyfill-webpack-plugin');
const path = require('path');
const webpack = require('webpack');
const { config } = require('dotenv');

config();
module.exports = {
	mode: 'development',
	entry: {
		main: './src/main.js',
	},
	output: {
		filename: 'bundle.js',
		path: path.resolve(__dirname, 'dist'),
	},
	devServer: {
		port: 8081,
	},
	plugins: [
		new HtmlWebpackPlugin({
			template: 'index.html',
			chunks: ['main'],
		}),
		new NodePolyfillPlugin({
			excludeAliases: ['console'],
		}),
		new webpack.DefinePlugin({
			'process.env.MNEMONIC_MAIN': JSON.stringify(process.env.MNEMONIC_MAIN),
		}),
	],
	resolve: {
		fallback: {
			path: require.resolve('path-browserify'),
			stream: require.resolve('stream-browserify'),
			crypto: require.resolve('crypto-browserify'),
		},
	},
	optimization: {
		minimize: true,
	},
};

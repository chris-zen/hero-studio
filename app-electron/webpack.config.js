const path = require('path')
const webpack = require('webpack')
const nodeExternals = require('webpack-node-externals')
const HtmlWebpackPlugin = require('html-webpack-plugin')
const HtmlWebpackRootPlugin = require('html-webpack-root-plugin')
const CspHtmlWebpackPlugin = require('csp-html-webpack-plugin')
const CspPolicy = {
  'base-uri': "'self'",
  'object-src': "'none'",
  'script-src': ["'unsafe-inline'", "'self'", "'unsafe-eval'"],
  'style-src': ["'unsafe-inline'", "'self'", "'unsafe-eval'"]
}
const MiniCssExtractPlugin = require('mini-css-extract-plugin')
// const CopyWebpackPlugin = require('copy-webpack-plugin')
// const HtmlWebpackIncludeAssetsPlugin = require('html-webpack-include-assets-plugin')
// const GoogleFontsPlugin = require('@beyonk/google-fonts-webpack-plugin')

const commonConfig = {
  // watch: true,
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js'
  },
  node: {
    __filename: false,
    __dirname: false
  },
  module: {
    rules: [
      {
        test: /\.ts$/,
        enforce: 'pre',
        loader: 'tslint-loader',
        options: {
          typeCheck: true,
          emitErrors: true
        }
      },
      {
        test: /\.tsx?$/,
        use: ['babel-loader', 'ts-loader']
      },
      {
        test: /\.js$/,
        enforce: 'pre',
        loader: 'standard-loader',
        options: {
          typeCheck: true,
          emitErrors: true
        }
      },
      {
        test: /\.jsx?$/,
        loader: 'babel-loader'
      },
      {
        test: /\.s?css$/,
        use: [
          // 'style-loader',
          MiniCssExtractPlugin.loader,
          'css-loader',
          'sass-loader'
        ]
      },
      {
        test: /\.(woff(2)?|ttf|eot)(\?v=\d+\.\d+\.\d+)?$/,
        use: [
          {
            loader: 'file-loader',
            options: {
              name: '[name].[ext]',
              outputPath: 'fonts/'
            }
          }
        ]
      },
      {
        test: /\.html$/,
        loader: 'file-loader',
        options: {
          name: '[name].[ext]'
        }
      },
      {
        test: /\.svg$/,
        // loader: 'svg-inline-loader'
        use: [
          {
            loader: 'babel-loader'
          },
          {
            loader: 'react-svg-loader',
            options: {
              jsx: true // true outputs JSX tags
            }
          }
        ]
      }
    ]
  },
  resolve: {
    extensions: [
      '.js', '.ts', '.tsx', '.jsx', '.json', '.scss', '.css', '.html', '.svg'
    ]
  }
}

module.exports = [
  Object.assign(
    {
      target: 'electron-main',
      entry: { main: './src/main.ts' },
      externals: [nodeExternals()]
    },
    commonConfig),
  Object.assign(
    {
      target: 'electron-renderer',
      entry: { gui: './src/gui.tsx' },
      plugins: [
        new HtmlWebpackPlugin({ title: 'Hero Studio' }),
        new HtmlWebpackRootPlugin(),
        new CspHtmlWebpackPlugin(CspPolicy),
        new MiniCssExtractPlugin(),
        // new CopyWebpackPlugin([
        //   { from: 'node_modules/normalize.css/normalize.css', to: 'blueprint/' },
        //   { from: 'node_modules/@blueprintjs/core/lib/css/blueprint.css', to: 'blueprint/' },
        //   { from: 'node_modules/@blueprintjs/icons/lib/css/blueprint-icons.css', to: 'blueprint/' }
        // ]),
        // new HtmlWebpackIncludeAssetsPlugin({
        //   assets: [
        //     'blueprint/normalize.css',
        //     'blueprint/blueprint.css',
        //     'blueprint/blueprint-icons.css'
        //   ],
        //   append: true
        // }),
        // new GoogleFontsPlugin({
        //   path: 'fonts/',
        //   fonts: [
        //     // { family: 'Source Sans Pro' },
        //     { family: 'Roboto', variants: [ '300', '400', '500' ] }
        //   ]
        // }),
        new webpack.DefinePlugin({
          'process.env': {
            NODE_ENV: '"development"'
          },
          'global': {
            GENTLY: false
          } // bizarre lodash(?) webpack workaround
        })
      ]
    },
    commonConfig)
]

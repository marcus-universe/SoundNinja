module.exports = {
  pluginOptions: {
    electronBuilder: {
      nodeIntegration: true,
      nodeisoation: false,
      outputDir: 'soundninja-winx64',
      appId: 'com.soundninja.soundninja',
      files: ["./config.json",
      {
        from: "./",
        to: "./",
        filter: ["**/*"], // This will recursively include all sub-directories & files
      },

    ],
      win: {
        icon: './icon.ico'
      }
    }
  },
};